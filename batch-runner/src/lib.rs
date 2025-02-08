use std::io::{Read, Seek};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum CompressAlgo {
    Zip,
    Tar,
    Gzip,
    Xz,
    Bzip2,
    Invalid,
}

pub fn get_compression_type(file_path: &PathBuf) -> CompressAlgo {
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| {
            format!(
                "Could not open: {:?} {} at {} {}",
                file_path,
                e,
                line!(),
                file!()
            )
        })
        .unwrap();

    let mut magic = [0u8; 6];
    let metadata = file
        .metadata()
        .map_err(|e| {
            format!(
                "Could not get metadata for: {:?} {} at {} {}",
                file_path,
                e,
                line!(),
                file!()
            )
        })
        .unwrap();

    if metadata.len() < 6 {
        return CompressAlgo::Invalid;
    }

    file.read_exact(&mut magic)
        .map_err(|e| format!("Could not read: {} at {} {}", e, line!(), file!()))
        .unwrap();

    let compression_type = if magic.starts_with(&[0x1F, 0x8B]) {
        CompressAlgo::Gzip
    } else if magic.starts_with(&[0x42, 0x5A, 0x68]) {
        CompressAlgo::Bzip2
    } else if magic.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
        CompressAlgo::Xz
    } else if magic.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        CompressAlgo::Zip
    } else {
        let mut magic = [0u8; 5];

        file.seek(std::io::SeekFrom::Current(251))
            .map_err(|e| format!("seek error: {} at {} {}", e, line!(), file!()))
            .unwrap();

        file.read_exact(&mut magic)
            .map_err(|e| format!("Could not read: {} at {} {}", e, line!(), file!()))
            .unwrap();

        if magic.starts_with(&[0x75, 0x73, 0x74, 0x61, 0x72]) {
            CompressAlgo::Tar
        } else {
            CompressAlgo::Invalid
        }
    };

    compression_type
}

pub fn get_mime(file_path: &PathBuf) -> Result<&str, Box<dyn std::error::Error>> {
    let mime = infer::get_from_path(file_path)?;
    match mime {
        Some(mime) => Ok(mime.mime_type()),
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not determine MIME type",
        ))),
    }
}

pub fn unzip(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(src)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = match entry.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if entry_path.starts_with("__MACOSX") || entry.is_dir() {
            continue;
        }

        let entry_path = entry_path.file_name().unwrap();

        let mut outfile = std::fs::File::create(dst.join(&entry_path))?;
        std::io::copy(&mut entry, &mut outfile)?;

        if let Some(mode) = entry.unix_mode() {
            std::fs::set_permissions(&dst.join(entry_path), std::fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

pub fn untar(
    src: &PathBuf,
    dst: &PathBuf,
    algorithm: CompressAlgo,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(src)?;
    let decoder: Box<dyn Read> = match algorithm {
        CompressAlgo::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
        CompressAlgo::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
        CompressAlgo::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        _ => Box::new(file),
    };

    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let entry_path = entry.path().unwrap();
        if entry_path.starts_with("__MACOSX") || entry.header().entry_type().is_dir() {
            continue;
        }
        let entry_filename = entry_path.file_name().unwrap();
        let mut outfile = std::fs::File::create(dst.join(&entry_filename))?;
        std::io::copy(&mut entry, &mut outfile)?;
    }
    Ok(())
}
