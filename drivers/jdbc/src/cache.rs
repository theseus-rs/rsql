use ristretto_resolver::{
    ArtifactCoordinate, Classpath, FileCache, FileDestination, RemoteRepository, ResolutionRequest,
    ResolutionRoot, Resolver, UpdatePolicy,
};
use rsql_driver::{Error, Result};
use std::path::{Path, PathBuf};

pub(crate) async fn driver_classpath(dependencies: &[ArtifactCoordinate]) -> Result<Vec<PathBuf>> {
    if dependencies.is_empty() {
        return Ok(Vec::new());
    }
    let cache = dirs::cache_dir()
        .ok_or_else(|| Error::IoError("Cannot locate the user cache directory".into()))?
        .join("rsql")
        .join("maven");
    let mut repository = RemoteRepository::central();
    // Reuse immutable release POMs and artifacts across sessions.
    repository.releases.update = UpdatePolicy::Never;
    let resolver = Resolver::builder()
        .repositories(vec![repository])
        .file_cache(FileCache::new(cache.join("repository")))
        .build()
        .map_err(resolver_error)?;
    resolve_classpath(&resolver, &cache.join("artifacts"), dependencies).await
}

fn resolver_error(error: impl std::fmt::Display) -> Error {
    Error::IoError(format!("JDBC Maven resolution: {error}"))
}

async fn resolve_classpath(
    resolver: &Resolver,
    destination: &Path,
    dependencies: &[ArtifactCoordinate],
) -> Result<Vec<PathBuf>> {
    let Some((first, rest)) = dependencies.split_first() else {
        return Ok(Vec::new());
    };
    // Resolver instances share repository and artifact files. Serialize their writes
    // across connections and processes, without blocking an async worker thread.
    let cache_directory = destination.to_path_buf();
    let _cache_lock = tokio::task::spawn_blocking(move || -> std::io::Result<std::fs::File> {
        std::fs::create_dir_all(&cache_directory)?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(cache_directory.join(".lock"))?;
        lock.lock()?;
        Ok(lock)
    })
    .await
    .map_err(resolver_error)?
    .map_err(resolver_error)?;
    let mut request = ResolutionRequest::new(first.clone()).with_classpath(Classpath::Runtime);
    for dependency in rest {
        request = request.with_root(ResolutionRoot::Artifact(dependency.clone()));
    }
    let destination = tokio::fs::canonicalize(destination).await?;
    let mut resolution = resolver.resolve(&request).await.map_err(resolver_error)?;
    let mut classpath = Vec::with_capacity(resolution.artifacts.len());
    let mut missing = Vec::new();
    for artifact in &resolution.artifacts {
        let path = destination.join(
            artifact
                .coordinate
                .artifact_path_with_version(&artifact.file_version),
        );
        // Replacing a JAR already opened by a JVM fails on Windows. Versioned
        // releases are immutable; only missing files and mutable snapshots need writes.
        if artifact.file_version.ends_with("-SNAPSHOT") || !cached_artifact(&path).await? {
            missing.push(artifact.clone());
        }
        classpath.push((artifact.node, path));
    }
    resolution.artifacts = missing;
    let report = resolver
        .download(&resolution, &FileDestination::new(destination))
        .await
        .map_err(resolver_error)?;
    for downloaded in report.artifacts {
        // Repository fallback can select a different timestamped snapshot path.
        if let Some((_, path)) = classpath
            .iter_mut()
            .find(|(node, _)| *node == downloaded.artifact.node)
        {
            *path = downloaded.output;
        }
    }
    Ok(classpath.into_iter().map(|(_, path)| path).collect())
}

async fn cached_artifact(path: &Path) -> Result<bool> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => {
            if !metadata.is_file() || tokio::fs::canonicalize(path).await? != path {
                return Err(Error::IoError(format!(
                    "Invalid JDBC cached artifact: {}",
                    path.display()
                )));
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

    #[tokio::test]
    async fn concurrent_resolutions_share_cache_and_reuse_it_offline() -> Result<()> {
        let server = MockServer::start().await;
        let root = "test/driver/1.0/driver-1.0";
        let pom = "<project><modelVersion>4.0.0</modelVersion><groupId>test</groupId><artifactId>driver</artifactId><version>1.0</version><dependencies><dependency><groupId>test</groupId><artifactId>dependency</artifactId><version>1.0</version><scope>runtime</scope></dependency></dependencies></project>";
        let dependency_pom = "<project><modelVersion>4.0.0</modelVersion><groupId>test</groupId><artifactId>dependency</artifactId><version>1.0</version></project>";
        for (name, body) in [
            (format!("{root}.pom"), pom.into()),
            (format!("{root}.jar"), "driver jar".into()),
            (
                "test/dependency/1.0/dependency-1.0.pom".into(),
                dependency_pom.into(),
            ),
            (
                "test/dependency/1.0/dependency-1.0.jar".into(),
                "dependency jar".into(),
            ),
            (
                "test/extra/1.0/extra-1.0.pom".into(),
                dependency_pom.replace("dependency", "extra"),
            ),
            ("test/extra/1.0/extra-1.0.jar".into(), "extra jar".into()),
        ] {
            Mock::given(path(format!("/{name}")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_string(body)
                        .set_delay(std::time::Duration::from_millis(50)),
                )
                .expect(1)
                .mount(&server)
                .await;
        }
        let directory = tempfile::tempdir()?;
        let mut repository = RemoteRepository::new("test", server.uri()).map_err(resolver_error)?;
        repository.releases.update = UpdatePolicy::Never;
        let cache = FileCache::new(directory.path().join("repository"));
        let resolver = Resolver::builder()
            .repositories(vec![repository.clone()])
            .file_cache(cache.clone())
            .build()
            .map_err(resolver_error)?;
        let dependencies = crate::Options::parse(
            "jdbc:test:db?dependency=test:driver:1.0&dependency=test:extra:1.0&dependency=test:driver:1.0",
        )?.dependencies;
        let other_resolver = Resolver::builder()
            .repositories(vec![repository.clone()])
            .file_cache(cache.clone())
            .build()
            .map_err(resolver_error)?;
        let destination = directory.path().join("artifacts");
        let (first, second) = tokio::join!(
            resolve_classpath(&resolver, &destination, &dependencies),
            resolve_classpath(&other_resolver, &destination, &dependencies),
        );
        let classpath = first?;
        assert_eq!(classpath, second?);
        assert_eq!(classpath.len(), 3);
        let jar = std::fs::OpenOptions::new().read(true).write(true).open(
            classpath
                .first()
                .ok_or_else(|| Error::IoError("Missing driver JAR".into()))?,
        )?;
        let modified = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1);
        jar.set_modified(modified)?;
        assert_eq!(
            std::fs::read(
                classpath
                    .first()
                    .ok_or_else(|| Error::IoError("Missing driver JAR".into()))?
            )?,
            b"driver jar"
        );
        assert_eq!(
            resolve_classpath(
                &resolver,
                &directory.path().join("artifacts"),
                &dependencies
            )
            .await?,
            classpath
        );
        let offline = Resolver::builder()
            .repositories(vec![repository])
            .file_cache(cache)
            .offline(true)
            .build()
            .map_err(resolver_error)?;
        let missing = classpath
            .last()
            .ok_or_else(|| Error::IoError("Missing JAR".into()))?;
        let contents = std::fs::read(missing)?;
        std::fs::remove_file(missing)?;
        assert_eq!(
            resolve_classpath(&offline, &directory.path().join("artifacts"), &dependencies).await?,
            classpath
        );
        assert_eq!(std::fs::read(missing)?, contents);
        assert_eq!(std::fs::metadata(&classpath[0])?.modified()?, modified);
        Ok(())
    }
}
