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

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use chrono::naive::NaiveDateTime;

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

fn copy_int(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<i32, io::Error> {
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
    writer.write_all(&buf)?;
    if buf[0] > 0 {
        Ok(-res_signed)
    } else {
        Ok(res_signed)
    }
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

fn copy_string_opt(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<Option<Vec<u8>>, io::Error> {
    let len: i32 = copy_int(reader, writer)?;
    if len < 0 {
        return Ok(None);
    }
    if 0 == len {
        return Ok(Some(Vec::with_capacity(0usize)))
    }
    let mut vec: Vec<u8> = Vec::with_capacity(len as usize);
    for i in 0..len {
        vec.push(0u8);
    }
    reader.read_exact(vec.as_mut_slice())?;
    writer.write_all(vec.as_slice())?;
    Ok(Some(vec))
}

fn copy_string(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<String, io::Error> {
    let bin_opt = copy_string_opt(reader, writer)?;
    match bin_opt {
        Some(bin) => Ok(String::from_utf8_lossy(bin.as_slice()).to_string()),
        None => Err(io::Error::new(io::ErrorKind::Other, "String read failed"))
    }
}

fn print_bin_str(label: &str, bin_opt: Option<Vec<u8>>) {
    match bin_opt {
        Some(bin) => {
            let st = String::from_utf8_lossy(bin.as_slice()).to_string();
            println!("{}: {}", label, st);
        },
        None => {}
    }
}

fn copy_toc_entry(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    println!("========================================");
    let _dump_id = copy_int(reader, writer)?;
    let _had_dumper = copy_int(reader, writer)?;
    let _table_oid = copy_string_opt(reader, writer)?;
    let _catalog_oid = copy_string_opt(reader, writer)?;
    let _tag = copy_string_opt(reader, writer)?;
    print_bin_str("tag", _tag);
    let _description = copy_string_opt(reader, writer)?;
    print_bin_str("description", _description);
    let _section = copy_int(reader, writer)?;
    let _defn = copy_string_opt(reader, writer)?;
    print_bin_str("defn", _defn);
    let _drop_stmt = copy_string_opt(reader, writer)?;
    print_bin_str("drop", _drop_stmt);
    let _copy_stmt = copy_string_opt(reader, writer)?;
    print_bin_str("copy", _copy_stmt);
    let _namespace = copy_string_opt(reader, writer)?;
    print_bin_str("namespace", _namespace);
    let _tablespace = copy_string_opt(reader, writer)?;
    print_bin_str("namespace", _tablespace);
    let _tableam = copy_string_opt(reader, writer)?;
    print_bin_str("tableam", _tableam);
    let _owner = copy_string_opt(reader, writer)?;
    print_bin_str("owner", _owner);
    let _table_with_oids = copy_string_opt(reader, writer)?;
    let mut _deps: Vec<String> = Vec::new();
    loop {
        match copy_string_opt(reader, writer)? {
            Some(bin) => {
                print_bin_str("dep", Some(bin.clone()));
                let st = String::from_utf8_lossy(bin.as_slice()).to_string();
                _deps.push(st)
            },
            None => break
        }
    }
    let _filename = copy_string_opt(reader, writer)?;
    print_bin_str("filename", _filename);
    Ok(())
}

fn copy_header(reader: &mut BufReader<File>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    copy_magic(reader, writer)?;
    copy_version(reader, writer)?;
    copy_flags(reader, writer)?;
    let comp = copy_int(reader, writer)?;
    println!("{}", comp);
    let timestamp = copy_timestamp(reader, writer)?;
    println!("{}", timestamp);
    let _dbname = copy_string(reader, writer)?;
    let version_server = copy_string(reader, writer)?;
    println!("{}", version_server);
    let version_pgdump = copy_string(reader, writer)?;
    println!("{}", version_pgdump);
    Ok(())
}

pub fn rewrite_toc(dir_path: &str, dbname: &str) -> Result<(), io::Error> {
    let toc_src_path = Path::new(dir_path).join(Path::new("toc.dat"));
    let toc_dest_path = Path::new(dir_path).join(Path::new("toc_rewritten.dat"));
    let toc_src = File::open(toc_src_path)?;
    let mut reader = BufReader::new(toc_src);
    let dest_file = File::create(toc_dest_path)?;
    let mut writer = BufWriter::new(dest_file);

    copy_header(&mut reader, &mut writer)?;
    let toc_count = copy_int(&mut reader, &mut writer)?;
    println!("{}", toc_count);
    for i in 0..toc_count {
        copy_toc_entry(&mut reader, &mut writer)?;
    }

    Ok(())
}