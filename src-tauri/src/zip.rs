use anyhow::Result;
use async_zip::base::write::ZipFileWriter;
use async_zip::tokio::write::ZipFileWriter as TokioZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use log::debug;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
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
        "zip: creating file from main file: {}",
        input_file.to_string_lossy()
    );
    debug!("zip: output file: {}", output_zip.to_string_lossy());
    debug!("zip: related extensions: {:?}", related_extensions);

    let file = File::create(output_zip).await?;
    let mut writer: TokioZipFileWriter<File> = ZipFileWriter::with_tokio(file);

    let mut related_files: Vec<PathBuf> = Vec::from([input_file.to_path_buf()]);
    for ext in related_extensions {
        let related = input_file.with_extension(ext);
        if let Ok(exists) = related.try_exists() {
            if exists {
                related_files.push(related);
            }
        }
    }
    debug!("zip: related files: {:?}", related_files);

    for rf in &related_files {
        write_entry(rf, &mut writer).await?;
    }

    writer.close().await?;

    Ok(())
}

/// Write a file to a zip writer.
async fn write_entry(input_path: &Path, writer: &mut TokioZipFileWriter<File>) -> Result<()> {
    let mut input_file = File::open(input_path).await?;
    let input_file_size = input_file.metadata().await?.len() as usize;

    let filename = input_path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid filename"))?
        .to_string_lossy()
        .to_string();
    debug!("zip: adding file '{}'", filename);

    let mut buffer = Vec::with_capacity(input_file_size);
    input_file.read_to_end(&mut buffer).await?;
    drop(input_file);

    let builder = ZipEntryBuilder::new(filename.into(), Compression::Zstd);
    writer.write_entry_whole(builder, &buffer).await?;
    drop(buffer);

    Ok(())
}
