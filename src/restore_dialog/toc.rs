/*
 * Copyright 2023, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use chrono::naive::NaiveDateTime;

#[derive(Default, Debug, Clone)]
struct TocEntry {
    dump_id: i32,
    had_dumper: i32,
    table_oid: Option<Vec<u8>>,
    catalog_oid: Option<Vec<u8>>,
    tag: Option<Vec<u8>>,
    description: Option<Vec<u8>>,
    section: i32,
    defn: Option<Vec<u8>>,
    drop_stmt: Option<Vec<u8>>,
    copy_stmt: Option<Vec<u8>>,
    namespace: Option<Vec<u8>>,
    tablespace: Option<Vec<u8>>,
    tableam: Option<Vec<u8>>,
    owner: Option<Vec<u8>>,
    table_with_oids: Option<Vec<u8>>,
    deps: Vec<Vec<u8>>,
    filename: Option<Vec<u8>>,
}


fn copy_magic(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    let mut buf = [0u8; 5];
    reader.read_exact( &mut buf)?;
    if [b'P', b'G', b'D', b'M', b'P'] != buf {
        return Err(io::Error::new(io::ErrorKind::Other, "Magic check failure"))
    }
    writer.write_all(&buf)?;
    Ok(())
}

fn copy_version(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    let mut buf = [0u8; 3];
    reader.read_exact( &mut buf)?;
    if 1u8 != buf[0] && 14u8 != buf[1] {
        return Err(io::Error::new(io::ErrorKind::Other, "Version check failure"))
    }
    writer.write_all(&buf)?;
    Ok(())
}

fn copy_flags(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    let mut buf = [0u8; 3];
    reader.read_exact( &mut buf)?;
    if 4u8 != buf[0] {
        return Err(io::Error::new(io::ErrorKind::Other, "Int size check failed"))
    }
    if 8u8 != buf[1] {
        return Err(io::Error::new(io::ErrorKind::Other, "Offset check failed"))
    }
    if 3u8 != buf[2] {
        return Err(io::Error::new(io::ErrorKind::Other, "Format check failed"))
    }
    writer.write_all(&buf)?;
    Ok(())
}

// todo: int size
fn write_int(writer: &mut BufWriter<File>, val: i32) -> Result<(), io::Error> {
    let mut buf = [0u8; 5];
    let uval = if val >= 0 {
        buf[0] = 0;
        val as u32
    } else {
        buf[0] = 1;
        -val as u32
    };
    let uval_bytes = uval.to_le_bytes();
    for i in 0..uval_bytes.len() {
        buf[i + 1] = uval_bytes[i];
    }
    writer.write_all(&buf)?;
    Ok(())
}

fn read_int(reader: &mut BufReader<File>) -> Result<i32, io::Error> {
    let mut buf = [0u8; 5];
    reader.read_exact( &mut buf)?;
    let mut res: u32 = 0;
    let mut shift: u32 = 0;
    for i in 1..buf.len() {
        let bv: u8 = buf[i];
        let iv: u32 = (bv as u32) & 0xFF;
        if iv != 0 {
            res = res + (iv << shift);
        }
        shift += 8;
    }
    let res_signed = res as i32;
    if buf[0] > 0 {
        Ok(-res_signed)
    } else {
        Ok(res_signed)
    }
}

fn copy_int(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<i32, io::Error> {
    let val = read_int(reader)?;
    write_int(writer, val)?;
    Ok(val)
}

fn copy_timestamp(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<NaiveDateTime, io::Error> {
    use chrono::naive::NaiveDate;
    use chrono::naive::NaiveTime;
    let sec = copy_int(reader, writer)?;
    let min = copy_int(reader, writer)?;
    let hour = copy_int(reader, writer)?;
    let day = copy_int(reader, writer)?;
    let month = copy_int(reader, writer)?;
    let year = copy_int(reader, writer)?;
    let _is_dst = copy_int(reader, writer)?;
    let date = NaiveDate::from_ymd_opt(year + 1900, month as u32, day as u32)
        .ok_or(io::Error::new(io::ErrorKind::Other, "Invalid date"))?;
    let time = NaiveTime::from_hms_opt(hour as u32, min as u32, sec as u32)
        .ok_or(io::Error::new(io::ErrorKind::Other, "Invalid time"))?;
    Ok(NaiveDateTime::new(date, time))
}

fn read_string_opt(reader: &mut BufReader<File>) -> Result<Option<Vec<u8>>, io::Error> {
    let len: i32 = read_int(reader)?;
    if len < 0 {
        return Ok(None);
    }
    if 0 == len {
        return Ok(Some(Vec::with_capacity(0usize)))
    }
    let mut vec: Vec<u8> = Vec::with_capacity(len as usize);
    for _ in 0..len {
        vec.push(0u8);
    }
    reader.read_exact(vec.as_mut_slice())?;
    Ok(Some(vec))
}

fn write_string_opt(writer: &mut BufWriter<File>, opt: &Option<Vec<u8>>) -> Result<(), io::Error> {
    match opt {
        Some(bytes) => {
            write_int(writer, bytes.len() as i32)?;
            writer.write_all(bytes.as_slice())?;
        },
        None => {
            write_int(writer, -1 as i32)?;
        }
    };
    Ok(())
}

fn copy_string_opt(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<Option<Vec<u8>>, io::Error> {
    let opt = read_string_opt(reader)?;
    write_string_opt(writer, &opt)?;
    Ok(opt)
}

fn copy_string(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<String, io::Error> {
    let bin_opt = copy_string_opt(reader, writer)?;
    match bin_opt {
        Some(bin) => Ok(String::from_utf8_lossy(bin.as_slice()).to_string()),
        None => Err(io::Error::new(io::ErrorKind::Other, "String read failed"))
    }
}

fn binopt_to_string(bin_opt: &Option<Vec<u8>>) -> String {
    match bin_opt {
        Some(bin) => {
            String::from_utf8_lossy(bin.as_slice()).to_string()
        },
        None => "".to_string()
    }
}

#[allow(dead_code)]
fn print_bin_str(label: &str, bin_opt: &Option<Vec<u8>>) {
    match bin_opt {
        Some(bin) => {
            let st = String::from_utf8_lossy(bin.as_slice()).to_string();
            println!("{}: {}", label, st);
        },
        None => {}
    }
}

fn read_toc_entry(reader: &mut BufReader<File>) -> Result<TocEntry, io::Error> {
    let dump_id = read_int(reader)?;
    let had_dumper = read_int(reader)?;
    let table_oid = read_string_opt(reader)?;
    let catalog_oid = read_string_opt(reader)?;
    let tag = read_string_opt(reader)?;
    let description = read_string_opt(reader)?;
    let section = read_int(reader)?;
    let defn = read_string_opt(reader)?;
    let drop_stmt = read_string_opt(reader)?;
    let copy_stmt = read_string_opt(reader)?;
    let namespace = read_string_opt(reader)?;
    let tablespace = read_string_opt(reader)?;
    let tableam = read_string_opt(reader)?;
    let owner = read_string_opt(reader)?;
    let table_with_oids = read_string_opt(reader)?;
    let mut deps: Vec<Vec<u8>> = Vec::new();
    loop {
        match read_string_opt(reader)? {
            Some(bytes) => deps.push(bytes),
            None => break
        }
    }
    let filename = read_string_opt(reader)?;
    Ok(TocEntry {
        dump_id,
        had_dumper,
        table_oid,
        catalog_oid,
        tag,
        description,
        section,
        defn,
        drop_stmt,
        copy_stmt,
        namespace,
        tablespace,
        tableam,
        owner,
        table_with_oids,
        deps,
        filename,
    })
}

fn write_toc_entry(writer: &mut BufWriter<File>, te: &TocEntry) -> Result<(), io::Error> {
    write_int(writer, te.dump_id)?;
    write_int(writer, te.had_dumper)?;
    write_string_opt(writer, &te.table_oid)?;
    write_string_opt(writer, &te.catalog_oid)?;
    write_string_opt(writer, &te.tag)?;
    write_string_opt(writer, &te.description)?;
    write_int(writer, te.section)?;
    write_string_opt(writer, &te.defn)?;
    write_string_opt(writer, &te.drop_stmt)?;
    write_string_opt(writer, &te.copy_stmt)?;
    write_string_opt(writer, &te.namespace)?;
    write_string_opt(writer, &te.tablespace)?;
    write_string_opt(writer, &te.tableam)?;
    write_string_opt(writer, &te.owner)?;
    write_string_opt(writer, &te.table_with_oids)?;
    for bytes in &te.deps {
        write_string_opt(writer, &Some(bytes.clone()))?;
    }
    write_string_opt(writer, &None)?;
    write_string_opt(writer, &te.filename)?;
    Ok(())
}

fn copy_header(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    copy_magic(reader, writer)?;
    copy_version(reader, writer)?;
    copy_flags(reader, writer)?;
    let _comp = copy_int(reader, writer)?;
    let _timestamp = copy_timestamp(reader, writer)?;
    let _dbname = copy_string(reader, writer)?;
    let _version_server = copy_string(reader, writer)?;
    let _version_pgdump = copy_string(reader, writer)?;
    Ok(())
}

fn rewrite_table(dir_path_st: &str, filename: &str, orig_dbname: &str, dbname: &str) -> Result<(), io::Error> {
    let dir_path = Path::new(dir_path_st);
    let file_src_path = dir_path.join(filename);
    let file_dest_path = dir_path.join(format!("{}.rewritten", filename));
    {
        let reader = BufReader::new(File::open(&file_src_path)?);
        let mut writer = BufWriter::new(File::create(&file_dest_path)?);
        for ln in reader.lines() {
            let line = ln?;
            let mut parts_replaced: Vec<String> = Vec::new();
            for part in line.split('\t') {
                let val = if part.starts_with(orig_dbname) {
                    part.replace(orig_dbname, dbname)
                } else {
                    part.to_string()
                };
                parts_replaced.push(val);
            }
            let line_replaced = parts_replaced.join("\t");
            writer.write_all(line_replaced.as_bytes())?;
            writer.write_all("\n".as_bytes())?;
        }
    }
    let file_orig_path = dir_path.join(format!("{}.orig", filename));
    fs::rename(&file_src_path, &file_orig_path)?;
    fs::rename(&file_dest_path, &file_src_path)?;
    Ok(())
}

fn rewrite_dbname_in_tables(map: &HashMap<String, String>, dir_path: &str, orig_dbname: &str, dbname: &str) -> Result<(), io::Error> {
    let babelfish_authid_user_ext_filename = match map.get("babelfish_authid_user_ext") {
        Some(name) => name,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Table not found: babelfish_authid_user_ext"))
    };
    rewrite_table(dir_path, babelfish_authid_user_ext_filename, orig_dbname, dbname)?;

    let babelfish_function_ext_filename = match map.get("babelfish_function_ext") {
        Some(name) => name,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Table not found: babelfish_function_ext"))
    };
    rewrite_table(dir_path, babelfish_function_ext_filename, orig_dbname, dbname)?;

    let babelfish_namespace_ext_filename = match map.get("babelfish_namespace_ext") {
        Some(name) => name,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Table not found: babelfish_namespace_ext"))
    };
    rewrite_table(dir_path, babelfish_namespace_ext_filename, orig_dbname, dbname)?;

    let babelfish_sysdatabases_filename = match map.get("babelfish_sysdatabases") {
        Some(name) => name,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Table not found: babelfish_sysdatabases"))
    };
    rewrite_table(dir_path, babelfish_sysdatabases_filename, orig_dbname, dbname)?;
    Ok(())
}

fn replace_dbname(te: &TocEntry, opt: &Option<Vec<u8>>, orig_dbname: &str, dbname: &str, can_add_dot: bool) -> Option<Vec<u8>> {
    if opt.is_none() {
        return None;
    }
    let te_tag = binopt_to_string(&te.tag);
    let mut needle_dbo = format!("{}_dbo", orig_dbname);
    let mut replacement_dbo = format!("{}_dbo", dbname);
    let mut needle_db_owner = format!("{}_db_owner", orig_dbname);
    let mut replacement_db_owner = format!("{}_db_owner", dbname);
    let mut needle_guest = format!("{}_guest", orig_dbname);
    let mut replacement_guest = format!("{}_guest", dbname);
    if  can_add_dot &&
        te_tag != format!("{}_dbo", &orig_dbname) &&
        te_tag != format!("SCHEMA {}_dbo", &orig_dbname) &&
        te_tag != format!("{}_guest", &orig_dbname) &&
        te_tag != format!("SCHEMA {}_guest", &orig_dbname)
    {
        needle_dbo.push('.');
        replacement_dbo.push('.');
        needle_db_owner.push('.');
        replacement_db_owner.push('.');
        needle_guest.push('.');
        replacement_guest.push('.');
    };
    let res = binopt_to_string(opt)
        .replace(&needle_dbo, &replacement_dbo)
        .replace(&needle_db_owner, &replacement_db_owner)
        .replace(&needle_guest, &replacement_guest);
    Some(res.into_bytes())
}

fn modify_toc_entry(te: &mut TocEntry, orig_dbname: &str, dbname: &str) {
    if orig_dbname.is_empty() {
        return;
    }
    te.defn = replace_dbname(&te, &te.defn, orig_dbname, dbname, true);
    te.copy_stmt = replace_dbname(&te, &te.copy_stmt, orig_dbname, dbname, true);
    te.drop_stmt = replace_dbname(&te, &te.drop_stmt, orig_dbname, dbname, true);
    te.namespace = replace_dbname(&te, &te.namespace, orig_dbname, dbname, false);
    te.owner = replace_dbname(&te, &te.owner, orig_dbname, dbname, false);
    // last
    te.tag = replace_dbname(&te, &te.tag, orig_dbname, dbname, false);
}

pub fn rewrite_toc(dir_path_st: &str, dbname: &str) -> Result<(), io::Error> {
    let dir_path = Path::new(dir_path_st);
    let toc_src_path = dir_path.join(Path::new("toc.dat"));
    let toc_dest_path = dir_path.join(Path::new("toc_rewritten.dat"));
    let toc_src = File::open(&toc_src_path)?;
    let mut reader = BufReader::new(toc_src);
    let dest_file = File::create(&toc_dest_path)?;
    let mut writer = BufWriter::new(dest_file);

    copy_header(&mut reader, &mut writer)?;
    let toc_count = copy_int(&mut reader, &mut writer)?;
    println!("{}", toc_count);
    let mut map: HashMap<String, String> = HashMap::new();
    let mut orig_dbname = "".to_string();
    for _ in 0..toc_count {
        let mut te  = read_toc_entry(&mut reader)?;
        let te_tag = binopt_to_string(&te.tag);
        let te_description = binopt_to_string(&te.description);
        let te_filename = binopt_to_string(&te.filename);
        if te_tag.ends_with("_dbo") && te_description == "SCHEMA" {
            orig_dbname = te_tag.chars().take(te_tag.len() - "_dbo".len()).collect();
        }
        if !te_filename.is_empty() {
            map.insert(te_tag, te_filename);
        }
        modify_toc_entry(&mut te, &orig_dbname, dbname);
        write_toc_entry(&mut writer, &te)?;
    }
    rewrite_dbname_in_tables(&map, dir_path_st, &orig_dbname, dbname)?;

    let toc_orig_path = dir_path.join("toc.dat.orig");
    fs::rename(&toc_src_path, &toc_orig_path)?;
    fs::rename(&toc_dest_path, &toc_src_path)?;


    Ok(())
}