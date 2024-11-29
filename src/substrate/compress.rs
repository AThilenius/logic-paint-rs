#[cfg(test)]
mod tests {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use tiff::encoder::compression::{Deflate, DeflateLevel, Lzw, Packbits};
    use tiff::encoder::{colortype, TiffEncoder};

    use std::io::{Cursor, Write};
    use std::time::Instant;

    use crate::{log, utils::convert::import_legacy_blueprint};

    #[test]
    pub fn compare_sizes() {
        let now = Instant::now();
        v1_json();
        println!("{} ms\n", now.elapsed().as_millis());

        // let now = Instant::now();
        // v1_bincode();
        // println!("{} ms\n", now.elapsed().as_millis());
        //
        // let now = Instant::now();
        // v2_bincode();
        // println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        tiff_rgba_lzw();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        tiff_rgba_deflate();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        tiff_rgba_packbits();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        tiff_gray_gray_deflate();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        deflate_one_channel();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        deflate_two_channel();
        println!("{} ms\n", now.elapsed().as_millis());
        let now = Instant::now();
        brotli_one_channel();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        brotli_one_channel();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        brotli_two_channel();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        brotli_one_channel_rgba_chunked();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        brotli_one_channel_rg_chunked();
        println!("{} ms\n", now.elapsed().as_millis());

        // let now = Instant::now();
        // brotli_over_v2();
        // println!("{} ms\n", now.elapsed().as_millis());

        for l in 0..12 {
            let now = Instant::now();
            brotli_one_channel_at_level(l);
            println!("{} ms\n", now.elapsed().as_millis());
        }

        let now = Instant::now();
        snappy_one_channel_rg_chunked();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        snappy_two_channel();
        println!("{} ms\n", now.elapsed().as_millis());

        let now = Instant::now();
        snappy_two_channel_chunked();
        println!("{} ms\n", now.elapsed().as_millis());
    }

    fn v1_json() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        log!("V1 Json:\t{}", json.len());
    }

    // fn v1_bincode() {
    //     let json = include_str!("../../misc/cpu.lpbp").to_string();
    //     let buffer = import_legacy_blueprint(json).unwrap();
    //     let bytes =
    //         bincode::encode_to_vec(EncodeV1::from(&buffer), bincode::config::standard()).unwrap();
    //     log!("V1 Bincode:\t{}", bytes.len());
    // }
    //
    // fn v2_bincode() {
    //     let json = include_str!("../../misc/cpu.lpbp").to_string();
    //     let buffer = import_legacy_blueprint(json).unwrap();
    //     let bytes =
    //         bincode::encode_to_vec(EncodeV2::from(&buffer), bincode::config::standard()).unwrap();
    //     log!("V2 Bincode:\t{}", bytes.len());
    // }

    fn tiff_rgba_lzw() {
        let image_data = buffer_as_single_image();

        let mut file = Cursor::new(Vec::new());
        let mut tiff = TiffEncoder::new(&mut file).unwrap();

        let image = tiff
            .new_image_with_compression::<colortype::RGBA8, Lzw>(
                image_data.width,
                image_data.height,
                Lzw::default(),
            )
            .unwrap();

        image.write_data(&image_data.pixels_rgba).unwrap();
        log!("TIFF (RG)BA LZW:\t{}", file.get_ref().len());
    }

    fn tiff_rgba_deflate() {
        let image_data = buffer_as_single_image();

        let mut file = Cursor::new(Vec::new());
        let mut tiff = TiffEncoder::new(&mut file).unwrap();

        let image = tiff
            .new_image_with_compression::<colortype::RGBA8, Deflate>(
                image_data.width,
                image_data.height,
                Deflate::with_level(DeflateLevel::Best),
            )
            .unwrap();

        image.write_data(&image_data.pixels_rgba).unwrap();
        log!("TIFF (RG)BA Deflate:\t{}", file.get_ref().len());
    }

    fn tiff_gray_gray_deflate() {
        let image_data = buffer_as_single_image();
        let r_data = image_data
            .pixels_rgba
            .iter()
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();
        let g_data = image_data
            .pixels_rgba
            .iter()
            .skip(1)
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();

        let r_bytes = {
            let mut file = Cursor::new(Vec::new());
            let mut tiff = TiffEncoder::new(&mut file).unwrap();

            let image = tiff
                .new_image_with_compression::<colortype::Gray8, Deflate>(
                    image_data.width,
                    image_data.height,
                    Deflate::with_level(DeflateLevel::Best),
                )
                .unwrap();

            image.write_data(&r_data).unwrap();

            file.get_ref().len()
        };

        let g_bytes = {
            let mut file = Cursor::new(Vec::new());
            let mut tiff = TiffEncoder::new(&mut file).unwrap();

            let image = tiff
                .new_image_with_compression::<colortype::Gray8, Deflate>(
                    image_data.width,
                    image_data.height,
                    Deflate::with_level(DeflateLevel::Best),
                )
                .unwrap();

            image.write_data(&g_data).unwrap();

            file.get_ref().len()
        };

        log!("TIFF Gray-Gray Deflate:\t{}", r_bytes + g_bytes);
    }

    fn tiff_rgba_packbits() {
        let image_data = buffer_as_single_image();

        let mut file = Cursor::new(Vec::new());
        let mut tiff = TiffEncoder::new(&mut file).unwrap();

        let image = tiff
            .new_image_with_compression::<colortype::RGBA8, Packbits>(
                image_data.width,
                image_data.height,
                Packbits::default(),
            )
            .unwrap();

        image.write_data(&image_data.pixels_rgba).unwrap();
        log!("TIFF (RG)BA Packbits:\t{}", file.get_ref().len());
    }

    fn deflate_one_channel() {
        let image_data = buffer_as_single_image();
        let bytes = image_data
            .pixels_rgba
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 4 < 2)
            .map(|(_, &val)| val)
            .collect::<Vec<_>>();

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).unwrap();
        let bytes = encoder.finish().unwrap();

        log!("Deflate one-channel:\t{}", bytes.len());
    }

    fn deflate_two_channel() {
        let image_data = buffer_as_single_image();
        let r_data = image_data
            .pixels_rgba
            .iter()
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();
        let g_data = image_data
            .pixels_rgba
            .iter()
            .skip(1)
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();

        let r = {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&r_data).unwrap();
            encoder.finish().unwrap()
        };

        let g = {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&g_data).unwrap();
            encoder.finish().unwrap()
        };

        log!("Deflate two-channel:\t{}", r.len() + g.len());
    }

    fn brotli_one_channel() {
        let image_data = buffer_as_single_image();
        let bytes = image_data
            .pixels_rgba
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 4 < 2)
            .map(|(_, &val)| val)
            .collect::<Vec<_>>();

        let file = Cursor::new(Vec::new());
        let mut writer = brotli::CompressorWriter::new(file, 100000, 7, 22);
        writer.write_all(&bytes).unwrap();
        writer.flush().unwrap();

        log!("Brotli one-channel:\t{}", writer.get_ref().get_ref().len());
    }

    fn brotli_two_channel() {
        let image_data = buffer_as_single_image();
        let r_data = image_data
            .pixels_rgba
            .iter()
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();
        let g_data = image_data
            .pixels_rgba
            .iter()
            .skip(1)
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();

        let file = Cursor::new(Vec::new());
        let mut writer = brotli::CompressorWriter::new(file, 100000, 7, 22);
        writer.write_all(&r_data).unwrap();
        writer.write_all(&g_data).unwrap();
        writer.flush().unwrap();

        log!("Brotli two-channel:\t{}", writer.get_ref().get_ref().len());
    }

    fn brotli_one_channel_rgba_chunked() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let file = Cursor::new(Vec::new());
        let mut writer = brotli::CompressorWriter::new(file, 4096, 7, 22);

        for chunk in buffer.chunks.values() {
            writer.write_all(&chunk.cells).unwrap();
        }
        writer.flush().unwrap();

        log!(
            "Brotli one channel RGBA chunked:\t{}",
            writer.get_ref().get_ref().len()
        );
    }

    fn brotli_one_channel_rg_chunked() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let file = Cursor::new(Vec::new());
        let mut writer = brotli::CompressorWriter::new(file, 4096, 7, 22);

        for chunk in buffer.chunks.values() {
            writer
                .write_all(
                    &chunk
                        .cells
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % 4 < 2)
                        .map(|(_, &val)| val)
                        .collect::<Vec<_>>(),
                )
                .unwrap();
        }
        writer.flush().unwrap();

        log!(
            "Brotli one channel RG chunked:\t{}",
            writer.get_ref().get_ref().len()
        );
    }

    fn snappy_one_channel_rg_chunked() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let mut writer = snap::write::FrameEncoder::new(Vec::new());

        for chunk in buffer.chunks.values() {
            writer
                .write_all(
                    &chunk
                        .cells
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % 4 < 2)
                        .map(|(_, &val)| val)
                        .collect::<Vec<_>>(),
                )
                .unwrap();
        }
        writer.flush().unwrap();

        log!(
            "Snappy one channel RG chunked:\t{}",
            writer.into_inner().unwrap().len()
        );
    }

    fn snappy_two_channel() {
        let image_data = buffer_as_single_image();
        let r_data = image_data
            .pixels_rgba
            .iter()
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();
        let g_data = image_data
            .pixels_rgba
            .iter()
            .skip(1)
            .step_by(4)
            .cloned()
            .collect::<Vec<_>>();

        let mut writer = snap::write::FrameEncoder::new(Vec::new());
        writer.write_all(&r_data).unwrap();
        writer.write_all(&g_data).unwrap();
        writer.flush().unwrap();

        log!(
            "Snappy two channel:\t{}",
            writer.into_inner().unwrap().len()
        );
    }

    fn snappy_two_channel_chunked() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let mut writer = snap::write::FrameEncoder::new(Vec::new());

        for chunk in buffer.chunks.values() {
            writer
                .write_all(
                    &chunk
                        .cells
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % 4 < 2)
                        .map(|(_, &val)| val)
                        .collect::<Vec<_>>(),
                )
                .unwrap();
        }

        writer.flush().unwrap();

        log!(
            "Snappy two channel chunked:\t{}",
            writer.into_inner().unwrap().len()
        );
    }

    // fn brotli_over_v2() {
    //     let json = include_str!("../../misc/cpu.lpbp").to_string();
    //     let buffer = import_legacy_blueprint(json).unwrap();
    //     let bytes =
    //         bincode::encode_to_vec(EncodeV2::from(&buffer), bincode::config::standard()).unwrap();
    //
    //     let file = Cursor::new(Vec::new());
    //     let mut writer = brotli::CompressorWriter::new(file, 4096, 7, 22);
    //     writer.write_all(&bytes).unwrap();
    //     writer.flush().unwrap();
    //
    //     log!(
    //         "Brotli over Encode V2:\t{}",
    //         writer.get_ref().get_ref().len()
    //     );
    // }

    fn brotli_one_channel_at_level(level: u32) {
        let image_data = buffer_as_single_image();
        let bytes = image_data
            .pixels_rgba
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 4 < 2)
            .map(|(_, &val)| val)
            .collect::<Vec<_>>();

        let file = Cursor::new(Vec::new());
        let mut writer = brotli::CompressorWriter::new(file, bytes.len(), level, 22);
        writer.write_all(&bytes).unwrap();
        writer.flush().unwrap();

        log!(
            "Brotli one-channel at level {}:\t{}",
            level,
            writer.get_ref().get_ref().len()
        );
    }

    struct SingleImage {
        width: u32,
        height: u32,
        offset_x: i32,
        offset_y: i32,
        pixels_rgba: Vec<u8>,
    }

    fn buffer_as_single_image() -> SingleImage {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let x_min = buffer
            .chunks
            .keys()
            .map(|k| k.first_cell_coord().0.x)
            .min()
            .unwrap();
        let y_min = buffer
            .chunks
            .keys()
            .map(|k| k.first_cell_coord().0.y)
            .min()
            .unwrap();
        let x_max = buffer
            .chunks
            .keys()
            .map(|k| k.last_cell_coord().0.x)
            .max()
            .unwrap();
        let y_max = buffer
            .chunks
            .keys()
            .map(|k| k.last_cell_coord().0.y)
            .max()
            .unwrap();

        let width = (x_max - x_min).abs() as u32;
        let height = (y_max - y_min).abs() as u32;
        let offset_x = -x_min;
        let offset_y = -y_min;

        let mut pixels_rgba = vec![0_u8; (width * height * 4) as usize];

        for cell_y in y_min..y_max {
            for cell_x in x_min..x_max {
                let image_x = cell_x - x_min;
                let image_y = cell_y - y_min;
                let image_i = (image_y * (width as i32) + image_x) * 4;
                let cell = buffer.get_cell((cell_x, cell_y).into());

                pixels_rgba[image_i as usize + 0] = cell.0[0];
                pixels_rgba[image_i as usize + 1] = cell.0[1];
                pixels_rgba[image_i as usize + 2] = cell.0[2];
                pixels_rgba[image_i as usize + 3] = cell.0[3];
            }
        }

        SingleImage {
            width,
            height,
            offset_x,
            offset_y,
            pixels_rgba,
        }
    }
}
