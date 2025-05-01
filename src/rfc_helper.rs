use std::vec;
use log::log;
use crate::error::RfcErrorInfo;
use crate::RfcConnection;

pub struct ResultSet {
    pub rows: Vec<String>,
    pub count: usize,
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
        let mut rfc_read_table = self.connection.get_function("RFC_READ_TABLE").expect("RFC_READ_TABLE");
        {
            let query_table = rfc_read_table
                .get_mut_parameter("QUERY_TABLE")
                .ok_or(RfcErrorInfo::custom("unknown field QUERY_TABLE"))?;
            query_table.set_string(self.table)?;
        }
        {
            let offset = rfc_read_table
                .get_mut_parameter("ROWSKIPS")
                .ok_or(RfcErrorInfo::custom("unknown field ROWSKIPS"))?;
            offset.set_int(page.offset as i64)?;
        }
        {
            let size = rfc_read_table
                .get_mut_parameter("ROWCOUNT")
                .ok_or(RfcErrorInfo::custom("unknown field ROWCOUNT"))?;
            size.set_int(page.size as i64)?;
        }
        {
            let sort = rfc_read_table
                .get_mut_parameter("GET_SORTED")
                .ok_or(RfcErrorInfo::custom("unknown field GET_SORTED"))?;
            sort.set_int(1)?;
        }
        {
            let delimiter = rfc_read_table
                .get_mut_parameter("DELIMITER")
                .ok_or(RfcErrorInfo::custom("unknown field DELIMITER"))?;
            delimiter.set_string(self.delimiter)?;
        }
        if !self.criteria.is_empty() {
            let option = rfc_read_table
                .get_mut_parameter("OPTIONS")
                .ok_or(RfcErrorInfo::custom("unknown field OPTIONS"))?;
            option.append_rows(1)?;
            option.first_row();
            let idx_fieldname = option
                .get_field_index_by_name("TEXT")?;
            let fieldname = option
                .get_field_by_index(idx_fieldname)?;
            fieldname.set_string(self.criteria)?;
        }
        if !self.fields.is_empty(){
            let fields = rfc_read_table.get_mut_parameter("FIELDS")
                .ok_or(RfcErrorInfo::custom("unknown field FIELDNAME"))?;
            let idx_fieldname = fields
                .get_field_index_by_name("FIELDNAME")?;
            fields.append_rows(self.fields.len() as u32 + 1_u32)?;

            fields.first_row()?;
            let fieldname = fields.get_field_by_index(idx_fieldname)?;
            fieldname.set_string("MANDT")?;

            for field in self.fields.iter() {
                fields.next_row()?;
                let fieldname = fields.get_field_by_index(idx_fieldname)?;
                fieldname.set_string(field.to_uppercase().as_str())?;
            }
        } else {
            log::warn!("No field provided for RFC_READ_TABLE");
        }

        rfc_read_table.call()?;

        // https://www.sapdatasheet.org/abap/func/rfc_read_table.html
        let data = rfc_read_table.get_mut_parameter("DATA").ok_or(RfcErrorInfo::custom("unknown field DATA"))?;
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

        Ok(ResultSet{
            count: rows.len(),
            has_more: rows.len() >= page.size,
            rows,
        })
    }
}