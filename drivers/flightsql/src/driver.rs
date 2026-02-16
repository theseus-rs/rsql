use crate::metadata;
use crate::results::FlightSqlQueryResult;
use arrow_array::cast::__private::DataType;
use arrow_array::{
    ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array,
    Int32Array, Int64Array, RecordBatch, StringArray, UInt8Array, UInt16Array, UInt32Array,
    UInt64Array,
};
use arrow_flight::FlightInfo;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::sql::client::FlightSqlServiceClient;
use async_trait::async_trait;
use file_type::FileType;
use futures_util::TryStreamExt;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, ToSql, Value};
use std::collections::HashMap;
use std::string::ToString;
use std::sync::Arc;
use tonic::transport::Channel;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "flightsql"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    client: FlightSqlServiceClient<Channel>,
}

impl Connection {
    /// Creates a new connection to the `FlightSQL` database.
    ///
    /// # Errors
    /// if the connection to the database fails.
    pub async fn new(url: &str) -> Result<Connection> {
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let parameters: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let scheme = match parameters.get("scheme") {
            Some(scheme) => scheme,
            None => "https",
        };
        let host = parsed_url
            .host_str()
            .ok_or_else(|| InvalidUrl("Missing host".to_string()))?;
        let port = parsed_url.port().unwrap_or(31337);
        let connection_url = format!("{scheme}://{host}:{port}");
        let channel = Channel::from_shared(connection_url)
            .map_err(|error| IoError(error.to_string()))?
            .connect()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let service_client = FlightServiceClient::new(channel);
        let mut client = FlightSqlServiceClient::new_from_inner(service_client);
        let username = parsed_url.username();
        let password = parsed_url
            .password()
            .map(ToString::to_string)
            .unwrap_or_default();

        if !username.is_empty() {
            let _ = client
                .handshake(username, &password)
                .await
                .map_err(|error| IoError(error.to_string()))?;
        }

        let connection = Connection {
            url: url.to_string(),
            client,
        };
        Ok(connection)
    }

    /// Returns the connection client
    pub(crate) fn client_mut(&mut self) -> &mut FlightSqlServiceClient<Channel> {
        &mut self.client
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let mut query_result = self.query(sql, params).await?;
        let mut rows = 0;
        while let Some(_row) = query_result.next().await {
            rows += 1;
        }
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let mut statement = self
            .client
            .prepare(sql.to_string(), None)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        if !values.is_empty() {
            let batch = values_to_record_batch(&values)?;
            statement
                .set_parameters(batch)
                .map_err(|error| IoError(error.to_string()))?;
        }
        let flight_info = statement
            .execute()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let query_result =
            convert_flight_info_to_query_result(&mut self.client, &flight_info).await?;
        Ok(query_result)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }
}

fn values_to_record_batch(values: &[Value]) -> Result<RecordBatch> {
    let columns: Vec<(String, ArrayRef)> = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let name = format!("param{}", i + 1);
            let array: ArrayRef = match value {
                Value::Null => Arc::new(StringArray::from(vec![None::<&str>])),
                Value::Bool(v) => Arc::new(BooleanArray::from(vec![*v])),
                Value::I8(v) => Arc::new(Int8Array::from(vec![*v])),
                Value::I16(v) => Arc::new(Int16Array::from(vec![*v])),
                Value::I32(v) => Arc::new(Int32Array::from(vec![*v])),
                Value::I64(v) => Arc::new(Int64Array::from(vec![*v])),
                Value::U8(v) => Arc::new(UInt8Array::from(vec![*v])),
                Value::U16(v) => Arc::new(UInt16Array::from(vec![*v])),
                Value::U32(v) => Arc::new(UInt32Array::from(vec![*v])),
                Value::U64(v) => Arc::new(UInt64Array::from(vec![*v])),
                Value::F32(v) => Arc::new(Float32Array::from(vec![*v])),
                Value::F64(v) => Arc::new(Float64Array::from(vec![*v])),
                Value::String(v) => Arc::new(StringArray::from(vec![v.as_str()])),
                Value::Bytes(v) => Arc::new(BinaryArray::from_vec(vec![v.as_slice()])),
                _ => Arc::new(StringArray::from(vec![value.to_string().as_str()])),
            };
            (name, array)
        })
        .collect();
    RecordBatch::try_from_iter(columns).map_err(|error| IoError(error.to_string()))
}

/// Converts a `FlightInfo` object to a `QueryResult` by fetching the data
pub(crate) async fn convert_flight_info_to_query_result(
    client: &mut FlightSqlServiceClient<Channel>,
    flight_info: &FlightInfo,
) -> Result<Box<dyn QueryResult>> {
    let schema = flight_info
        .clone()
        .try_decode_schema()
        .map_err(|error| IoError(error.to_string()))?
        .clone();
    let columns = schema
        .fields
        .iter()
        .map(|field| field.name().to_string())
        .collect::<Vec<_>>();

    let Some(ticket) = flight_info.endpoint[0].ticket.clone() else {
        return Err(IoError("Ticket not present".to_string()));
    };

    let flight_data = client
        .do_get(ticket)
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let batches: Vec<RecordBatch> = flight_data
        .try_collect()
        .await
        .map_err(|error| IoError(error.to_string()))?;

    let field_types: Vec<DataType> = schema
        .fields
        .iter()
        .map(|field| field.data_type().clone())
        .collect();

    let query_result = FlightSqlQueryResult::new(columns, field_types, batches);
    Ok(Box::new(query_result))
}
