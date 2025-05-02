extern crate rsrfc;

use std::any::Any;
use std::io::stdout;
use rsrfc::*;
use rsrfc::error::RfcErrorInfo;
use rsrfc::rfc_helper::{Page, RfcReadTable};

fn main() -> Result<(), RfcErrorInfo>{
    let conn_params = RfcConnectionParameters {
        ashost: "vhcala4hci",
        sysnr: "00",
        client: "000",
        user: "SAP*",
        passwd: "ABAPtr2022#01",
        lang: "EN",
    };

    // Open the rfc dll or .so
    let rfc_dll = RfcLib::new().expect("Unable to open the rfc lib");

    // Establish an RFC connection. If you need to supply more parameters than
    // those supported by RfcConnectionParameters, simply call
    // RfcConnection::from_hashmap instead.
    let conn = RfcConnection::new(&conn_params, &rfc_dll);
    let conn = match conn {
        Err(e) => {
            eprintln!("oops {:?}", e);
            return Err(e);
        }
        Ok(c) => c
    };

    let version = RfcReadTable::new(&conn, "MTC_PING").ping()?;
    

    eprintln!("Version {version}");

    let rfc_read_table = RfcReadTable::new(&conn, "USR02").fields(vec!["BNAME", "USTYP", "CLASS", "GLTGV", "GLTGB", "UFLAG"]).return_count(false);
    let resultset = rfc_read_table.fetch(Page::new())?;
    for row in resultset.rows {
        println!("---{}----", row.trim_end());
    }
    println!("{}", resultset.count);
    println!("{}", resultset.total);
    let rfc_read_table = RfcReadTable::new(&conn, "AGR_1251").fields(vec!["AGR_NAME", "OBJECT", "AUTH", "FIELD", "LOW", "HIGH", "DELETED"]).export_deleted_rows(true)
        // .criteria("DELETED <> 'X'")
        ;
    let mut has_more = true;
    let mut total = 0;
    let mut total_rs = 0;
    let mut iteration = 0;
    let size = 10000;
    while(has_more) {
        let resultset = rfc_read_table.fetch(Page::new().size(size).offset(iteration * size))?;
        for row in resultset.rows.iter() {
            println!("---{}----", row.trim_end());
        }
        total += resultset.count;
        iteration += 1;
        has_more = resultset.has_more;
        total_rs = resultset.total;
    }
    println!("total {}", total_rs);

    // {
    //     // Get the RFC_READ_TABLE function
    //     let mut rfc_read_table = conn.get_function("RFC_READ_TABLE").expect("RFC_READ_TABLE");
    //     {
    //         let query_table = rfc_read_table
    //             .get_mut_parameter("QUERY_TABLE")
    //             .ok_or(RfcErrorInfo::custom("unknown field QUERY_TABLE"))?;
    //         query_table.set_string("USR02")?;
    //     }
    //     {
    //         let delimiter = rfc_read_table
    //             .get_mut_parameter("DELIMITER")
    //             .ok_or(RfcErrorInfo::custom("unknown field DELIMITER"))?;
    //         delimiter.set_string("\t")?;
    //     }
    //     {
    //         let option = rfc_read_table
    //             .get_mut_parameter("OPTIONS")
    //             .ok_or(RfcErrorInfo::custom("unknown field OPTIONS"))?;
    //         option.append_rows(1)?;
    //         option.first_row();
    //         let idx_fieldname = option
    //             .get_field_index_by_name("TEXT")?;
    //         let fieldname = option
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname.set_string("BNAME LIKE 'S%' AND USTYP = 'A'")?;
    //     }
    //
    //     // The field we are interested in is called BNAME.
    //     // Tell this to the RFC_READ_TABLE function.
    //     {
    //         let fields = rfc_read_table.get_mut_parameter("FIELDS")
    //         .ok_or(RfcErrorInfo::custom("unknown field FIELDNAME"))?;
    //         let idx_fieldname = fields
    //             .get_field_index_by_name("FIELDNAME")?;
    //         fields.append_rows(6)?;
    //         fields.first_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("BNAME")?;
    //         fields.next_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("USTYP")?;
    //         fields.next_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("GLTGV")?;
    //         fields.next_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("GLTGB")?;
    //         fields.next_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("UFLAG")?;
    //         fields.next_row()?;
    //         let fieldname = fields
    //             .get_field_by_index(idx_fieldname)?;
    //         fieldname
    //             .set_string("CLASS")?;
    //     }
    //
    //     // Call the function
    //     rfc_read_table.call()?;
    //
    //     // Now the local data structures are filled with the response of the
    //     // remote function: retrieve the data
    //     let data = rfc_read_table.get_mut_parameter("DATA")
    //     .ok_or(RfcErrorInfo::custom("unknown field DATA"))?;
    //     // Get the intger index of the field to allow quicker access later
    //     let idx_wa = data.get_field_index_by_name("WA")?;
    //     let num_users = data.get_row_count()?;
    //     eprintln!(
    //         "Response from SAP has arrived: {} users.",
    //         num_users
    //     );
    //     for i in 0..num_users {
    //         data.set_row(i)?;
    //         let row_content = data
    //             .get_field_by_index(idx_wa)?
    //             .get_chars()?;
    //         println!("Username: {}", row_content.trim_end());
    //     }

    Ok(())
}
