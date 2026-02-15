use arrow_array::cast::__private::{DataType, TimeUnit};
use arrow_array::{
    ArrayRef, BinaryArray, BooleanArray, Date32Array, Date64Array, Float32Array, Float64Array,
    Int8Array, Int16Array, Int32Array, Int64Array, LargeStringArray, RecordBatch, StringArray,
    Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray, Time64NanosecondArray,
    TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
    TimestampSecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use async_trait::async_trait;
use jiff::ToSpan;
use jiff::civil::{Date, DateTime, Time};
use rsql_driver::Error::IoError;
use rsql_driver::{QueryResult, Result, Value};

const EPOCH_DATE: Date = Date::constant(1970, 1, 1);
const EPOCH_DATETIME: DateTime = DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

/// Query result that converts Arrow batches to values on demand
#[derive(Debug)]
pub(crate) struct FlightSqlQueryResult {
    columns: Vec<String>,
    field_types: Vec<DataType>,
    batches: Vec<RecordBatch>,
    batch_index: usize,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl FlightSqlQueryResult {
    pub(crate) fn new(
        columns: Vec<String>,
        field_types: Vec<DataType>,
        batches: Vec<RecordBatch>,
    ) -> Self {
        Self {
            columns,
            field_types,
            batches,
            batch_index: 0,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for FlightSqlQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        loop {
            if self.batch_index >= self.batches.len() {
                return None;
            }
            let batch = &self.batches[self.batch_index];
            if self.row_index >= batch.num_rows() {
                self.batch_index += 1;
                self.row_index = 0;
                continue;
            }
            self.row_buffer.clear();
            for (column_index, data_type) in self.field_types.iter().enumerate() {
                let column = batch.column(column_index);
                let value = convert_arrow_to_value(column, self.row_index, data_type).ok()?;
                self.row_buffer.push(value);
            }
            self.row_index += 1;
            return Some(&self.row_buffer);
        }
    }
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
    column: &'a std::sync::Arc<dyn arrow_array::Array + 'static>,
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
