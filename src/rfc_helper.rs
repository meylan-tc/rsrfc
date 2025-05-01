use std::vec;
use log::log;
use crate::error::RfcErrorInfo;
use crate::RfcConnection;

pub struct ResultSet {
    pub rows: Vec<String>,
    pub count: usize,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Copy, Clone)]
pub struct Page {
    size: usize,
    offset: usize
}

impl Page {
    pub fn new() -> Self {
        Self {
            size: 10000,
            offset: 0,
        }
    }
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

pub struct RfcReadTable<'a, 'conn> {
    table: &'a str,
    fields: Vec<&'a str>,
    delimiter: &'a str,
    criteria: &'a str,
    connection: &'conn RfcConnection<'conn>
}

impl <'a, 'conn> RfcReadTable<'a, 'conn> {
    pub fn new(connection: &'conn RfcConnection<'conn>, table: &'a str) -> Self {
        Self {
            table,
            fields: vec![],
            delimiter: "\t",
            criteria: "",
            connection,
        }
    }

    #[inline]
    pub fn fields(mut self, fields: Vec<&'a str>) -> Self {
        self.fields = fields;
        self
    }
    #[inline]
    pub fn delimiter(mut self, delimiter: &'a str) -> Self {
        self.delimiter = delimiter;
        self
    }
    #[inline]
    pub fn criteria(mut self, criteria: &'a str) -> Self {
        self.criteria = criteria;
        self
    }

    pub fn fetch(&self, page: Page) -> Result<ResultSet, RfcErrorInfo> {
        // https://www.sapdatasheet.org/abap/func/rfc_read_table.html
        let mut rfc_read_table = self.connection.get_function("Z_MTC_TABLE_READER").expect("Z_MTC_TABLE_READER");
        {
            let query_table = rfc_read_table
                .get_mut_parameter("IV_TABLE_NAME")
                .ok_or(RfcErrorInfo::custom("unknown field QUERY_TABLE"))?;
            query_table.set_string(self.table)?;
        }
        {
            let offset = rfc_read_table
                .get_mut_parameter("IV_OFFSET")
                .ok_or(RfcErrorInfo::custom("unknown field IV_OFFSET"))?;
            offset.set_int(page.offset as i64)?;
        }
        {
            let size = rfc_read_table
                .get_mut_parameter("IV_LIMIT")
                .ok_or(RfcErrorInfo::custom("unknown field IV_LIMIT"))?;
            size.set_int(page.size as i64)?;
        }
        {
            let delimiter = rfc_read_table
                .get_mut_parameter("IV_DELIMITER")
                .ok_or(RfcErrorInfo::custom("unknown field IV_DELIMITER"))?;
            delimiter.set_string(self.delimiter)?;
        }
        if !self.fields.is_empty(){
            let fields = rfc_read_table.get_mut_parameter("IT_FIELDS")
                .ok_or(RfcErrorInfo::custom("unknown field FIELDNAME"))?;
            let idx_fieldname = 0;
            fields.append_rows(self.fields.len() as u32 + 1_u32)?;

            fields.first_row()?;
            let fieldname = fields.get_field_by_index(idx_fieldname)?;
            fieldname.set_string("MANDT")?;

            for (i, field) in self.fields.iter().enumerate() {
                // if i == 0 {
                //     continue;
                // }
                fields.next_row()?;
                let fieldname = fields.get_field_by_index(idx_fieldname)?;
                fieldname.set_string(field.to_uppercase().as_str())?;
            }
        } else {
            log::warn!("No field provided for RFC_READ_TABLE");
        }

        rfc_read_table.call()?;

        // https://www.sapdatasheet.org/abap/func/rfc_read_table.html
        let data = rfc_read_table.get_mut_parameter("ET_DATA").ok_or(RfcErrorInfo::custom("unknown field DATA"))?;
        // WA is the single field of DATA, it ease a generic sap field name, meaning "Work Area"
        // https://www.sapdatasheet.org/abap/tabl/tab512.html
        let idx_wa = data.get_field_index_by_name("WA")?;

        let rows_count = data.get_row_count()?;
        let mut rows = Vec::with_capacity(rows_count as usize);
            for i in 0..rows_count {
            data.set_row(i)?;
            let row_content = data
                .get_field_by_index(idx_wa)?
                .get_chars()?;
            rows.push(row_content);
        }
        let total_count = rfc_read_table.get_mut_parameter("EV_TOTAL_COUNT").ok_or(RfcErrorInfo::custom("unknown field EV_TOTAL_COUNT"))?;
        
        Ok(ResultSet{
            count: rows.len(),
            total: total_count.get_int()? as usize,
            has_more: rows.len() >= page.size,
            rows,
        })
    }
}