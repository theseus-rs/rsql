use crate::metadata;
use arrow_array::cast::__private::{DataType, TimeUnit};
use arrow_array::{
    ArrayRef, BinaryArray, BooleanArray, Date32Array, Date64Array, Float32Array, Float64Array,
    Int8Array, Int16Array, Int32Array, Int64Array, LargeStringArray, StringArray,
    Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray, Time64NanosecondArray,
    TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
    TimestampSecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow_flight::FlightInfo;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::sql::client::FlightSqlServiceClient;
use async_trait::async_trait;
use file_type::FileType;
use futures_util::TryStreamExt;
use jiff::ToSpan;
use jiff::civil::{Date, DateTime, Time};
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, Value};
use std::collections::HashMap;
use std::string::ToString;
use tonic::transport::Channel;
use url::Url;

const EPOCH_DATE: Date = Date::constant(1970, 1, 1);
const EPOCH_DATETIME: DateTime = DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

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
        let service_client = FlightServiceClient::connect(connection_url)
            .await
            .map_err(|error| IoError(error.to_string()))?;
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

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let mut query_result = self.query(sql).await?;
        let mut rows = 0;
        while let Some(_row) = query_result.next().await {
            rows += 1;
        }
        Ok(rows)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut statement = self
            .client
            .prepare(sql.to_string(), None)
            .await
            .map_err(|error| IoError(error.to_string()))?;
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
    let batches: Vec<_> = flight_data
        .try_collect()
        .await
        .map_err(|error| IoError(error.to_string()))?;

    let mut rows = Vec::new();
    for batch in batches {
        let number_of_rows = batch.num_rows();
        for row_index in 0..number_of_rows {
            let mut row = Vec::new();
            for (column_index, field) in schema.fields.iter().enumerate() {
                let column = batch.column(column_index);
                let value = convert_arrow_to_value(column, row_index, field.data_type())?;
                row.push(value);
            }
            rows.push(row);
        }
    }

    let query_result = MemoryQueryResult::new(columns, rows);
    Ok(Box::new(query_result))
}

/// Converts an Arrow array value at the specified row index to the appropriate rsql Value type
#[expect(clippy::too_many_lines)]
fn convert_arrow_to_value(
    column: &ArrayRef,
    row_index: usize,
    data_type: &DataType,
) -> Result<Value> {
    if column.is_null(row_index) {
        return Ok(Value::Null);
    }

    let value = match data_type {
        DataType::Null => Value::Null,
        DataType::Binary => {
            let array = downcast_array::<BinaryArray>(column)?;
            let bytes = array.value(row_index);
            Value::Bytes(bytes.to_vec())
        }
        DataType::Boolean => {
            let array = downcast_array::<BooleanArray>(column)?;
            Value::Bool(array.value(row_index))
        }
        DataType::Int8 => {
            let array = downcast_array::<Int8Array>(column)?;
            Value::I8(array.value(row_index))
        }
        DataType::Int16 => {
            let array = downcast_array::<Int16Array>(column)?;
            Value::I16(array.value(row_index))
        }
        DataType::Int32 => {
            let array = downcast_array::<Int32Array>(column)?;
            Value::I32(array.value(row_index))
        }
        DataType::Int64 => {
            let array = downcast_array::<Int64Array>(column)?;
            Value::I64(array.value(row_index))
        }
        DataType::UInt8 => {
            let array = downcast_array::<UInt8Array>(column)?;
            Value::U8(array.value(row_index))
        }
        DataType::UInt16 => {
            let array = downcast_array::<UInt16Array>(column)?;
            Value::U16(array.value(row_index))
        }
        DataType::UInt32 => {
            let array = downcast_array::<UInt32Array>(column)?;
            Value::U32(array.value(row_index))
        }
        DataType::UInt64 => {
            let array = downcast_array::<UInt64Array>(column)?;
            Value::U64(array.value(row_index))
        }
        DataType::Float32 => {
            let array = downcast_array::<Float32Array>(column)?;
            Value::F32(array.value(row_index))
        }
        DataType::Float64 => {
            let array = downcast_array::<Float64Array>(column)?;
            Value::F64(array.value(row_index))
        }
        DataType::Utf8 => {
            let array = downcast_array::<StringArray>(column)?;
            Value::String(array.value(row_index).to_string())
        }
        DataType::LargeUtf8 => {
            let array = downcast_array::<LargeStringArray>(column)?;
            Value::String(array.value(row_index).to_string())
        }
        DataType::Date32 => {
            let array = downcast_array::<Date32Array>(column)?;
            let days_since_epoch = array.value(row_index).days();
            let date = EPOCH_DATE + days_since_epoch;
            Value::Date(date)
        }
        DataType::Date64 => {
            let array = downcast_array::<Date64Array>(column)?;
            let milliseconds_since_epoch = array.value(row_index).milliseconds();
            let date = EPOCH_DATE + milliseconds_since_epoch;
            Value::Date(date)
        }
        DataType::Time32(TimeUnit::Second) => {
            let array = downcast_array::<Time32SecondArray>(column)?;
            let time_seconds = array.value(row_index).seconds();
            let time = Time::MIN + time_seconds;
            Value::Time(time)
        }
        DataType::Time32(TimeUnit::Millisecond) => {
            let array = downcast_array::<Time32MillisecondArray>(column)?;
            let time_milliseconds = array.value(row_index).milliseconds();
            let time = Time::MIN + time_milliseconds;
            Value::Time(time)
        }
        DataType::Time64(TimeUnit::Microsecond) => {
            let array = downcast_array::<Time64MicrosecondArray>(column)?;
            let time_microseconds = array.value(row_index).microseconds();
            let time = Time::MIN + time_microseconds;
            Value::Time(time)
        }
        DataType::Time64(TimeUnit::Nanosecond) => {
            let array = downcast_array::<Time64NanosecondArray>(column)?;
            let time_nanoseconds = array.value(row_index).nanoseconds();
            let time = Time::MIN + time_nanoseconds;
            Value::Time(time)
        }
        DataType::Timestamp(TimeUnit::Second, _timezone) => {
            let array = downcast_array::<TimestampSecondArray>(column)?;
            let seconds_since_epoch = array.value(row_index).seconds();
            let datetime = EPOCH_DATETIME + seconds_since_epoch;
            // TODO: implement timezone handling
            Value::DateTime(datetime)
        }
        DataType::Timestamp(TimeUnit::Millisecond, _timezone) => {
            let array = downcast_array::<TimestampMillisecondArray>(column)?;
            let milliseconds_since_epoch = array.value(row_index).milliseconds();
            let datetime = EPOCH_DATETIME + milliseconds_since_epoch;
            // TODO: implement timezone handling
            Value::DateTime(datetime)
        }
        DataType::Timestamp(TimeUnit::Microsecond, _timezone) => {
            let array = downcast_array::<TimestampMicrosecondArray>(column)?;
            let microseconds_since_epoch = array.value(row_index).microseconds();
            let datetime = EPOCH_DATETIME + microseconds_since_epoch;
            // TODO: implement timezone handling
            Value::DateTime(datetime)
        }
        DataType::Timestamp(TimeUnit::Nanosecond, _timezone) => {
            let array = downcast_array::<TimestampNanosecondArray>(column)?;
            let nanseconds_since_epoch = array.value(row_index).nanoseconds();
            let datetime = EPOCH_DATETIME + nanseconds_since_epoch;
            // TODO: implement timezone handling
            Value::DateTime(datetime)
        }
        _ => {
            return Err(IoError(format!("Unsupported data type: {data_type:?}")));
        }
    };

    Ok(value)
}

/// Helper function to downcast an Arrow array to a specific type
fn downcast_array<'a, T>(
    column: &'a std::sync::Arc<(dyn arrow_array::Array + 'static)>,
) -> Result<&'a T>
where
    T: 'static + std::any::Any,
{
    let array_type_name = std::any::type_name::<T>();
    column
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| IoError(format!("Failed to downcast {array_type_name} array")))
}
