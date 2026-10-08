use anyhow::Result;
use async_zip::base::write::ZipFileWriter;
use async_zip::tokio::write::ZipFileWriter as TokioZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use log::debug;
use std::io::Error;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio_util::compat::FuturesAsyncWriteCompatExt;

/// Create a zip file containing the main file and any related files with the given extensions.
///
/// # Arguments
/// * `input_file` - The main file to include in the zip.
/// * `related_extensions` - The extensions of related files to include in the zip.
/// * `output_zip` - The path to the output zip file.
///
pub async fn related_files(
    input_file: &Path,
    related_extensions: &[&str],
    output_zip: &Path,
) -> Result<()> {
    debug!(
        "creating file from main file: {}",
        input_file.to_string_lossy()
    );
    debug!("output file: {}", output_zip.to_string_lossy());
    debug!("related extensions: {related_extensions:?}");

    let file = File::create(output_zip).await?;
    let mut writer: TokioZipFileWriter<File> = ZipFileWriter::with_tokio(file);

    let mut related_files: Vec<PathBuf> = Vec::from([input_file.to_path_buf()]);
    for ext in related_extensions {
        let related = input_file.with_extension(ext);
        if let Ok(exists) = related.try_exists()
            && exists
        {
            related_files.push(related);
        }
    }
    debug!("related files: {related_files:?}");

    for rf in &related_files {
        write_entry(rf, &mut writer).await?;
    }

    writer.close().await?;

    Ok(())
}

/// Write a file to a zip writer.
///
/// Entry bytes stream from disk through a fixed-size buffer, so backups
/// never hold a whole database file in memory.
async fn write_entry(input_path: &Path, writer: &mut TokioZipFileWriter<File>) -> Result<()> {
    let mut input_file = File::open(input_path).await?;

    let filename = input_path
        .file_name()
        .ok_or_else(|| Error::other("Invalid filename"))?
        .to_string_lossy()
        .to_string();
    debug!("adding file '{filename}'");

    let builder = ZipEntryBuilder::new(filename.into(), Compression::Zstd);
    let entry_writer = writer.write_entry_stream(builder).await?;
    let mut entry_compat = entry_writer.compat_write();
    tokio::io::copy(&mut input_file, &mut entry_compat).await?;
    drop(input_file);
    entry_compat.into_inner().close().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_zip::tokio::read::fs::ZipFileReader;

    #[tokio::test]
    async fn related_files_round_trip_streams_entries() {
        // GIVEN a main file plus related files larger than the copy buffer
        let dir = tempfile::tempdir().expect("tempdir");
        let main = dir.path().join("memos_prod.db");
        let wal = dir.path().join("memos_prod.db-wal");
        let shm = dir.path().join("memos_prod.db-shm");
        let main_bytes: Vec<u8> = (0..100_000_u32).map(|v| (v % 251) as u8).collect();
        let wal_bytes: Vec<u8> = (0..40_000_u32).map(|v| (v % 241) as u8).collect();
        tokio::fs::write(&main, &main_bytes)
            .await
            .expect("write main");
        tokio::fs::write(&wal, &wal_bytes).await.expect("write wal");
        tokio::fs::write(&shm, b"shm").await.expect("write shm");
        let output = dir.path().join("backup.zst.zip");

        // WHEN backing up the main file with related extensions
        related_files(&main, &["db-wal", "db-shm"], &output)
            .await
            .expect("backup");

        // THEN the archive holds byte-identical entries
        let reader = ZipFileReader::new(&output).await.expect("open zip");
        let names: Vec<String> = reader
            .file()
            .entries()
            .iter()
            .map(|entry| entry.filename().as_str().expect("name").to_string())
            .collect();
        assert_eq!(
            names,
            ["memos_prod.db", "memos_prod.db-wal", "memos_prod.db-shm"]
        );
        let expected = [&main_bytes, &wal_bytes, b"shm".as_slice()];
        for (index, want) in expected.into_iter().enumerate() {
            let mut entry = reader.reader_with_entry(index).await.expect("entry reader");
            let mut got = Vec::new();
            entry
                .read_to_end_checked(&mut got)
                .await
                .expect("read entry");
            assert_eq!(&got, want);
        }
    }
}
