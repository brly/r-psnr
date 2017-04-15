use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io;

extern crate getopts;
use getopts::Options;
use std::env;

mod common;

fn validate_option() -> Result<common::RPOption, String> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("r", "ref", "set input yuv (reference) file path", "NAME");
    opts.optopt("d", "dis", "set input yuv (distorted) file path", "NAME");
    opts.optopt("h", "height", "set input yuv height", "VALUE");
    opts.optopt("w", "wight", "set input yuv width", "VALUE");
    opts.optopt("m", "metric", "set metric", "NAME");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!(f.to_string()) }
    };

    // TODO error handling
    let ref_path = matches.opt_str("r").unwrap();
    let dis_path = matches.opt_str("d").unwrap();
    let height = matches.opt_str("h").unwrap().parse().unwrap();
    let width = matches.opt_str("w").unwrap().parse().unwrap();
    let metric = match matches.opt_str("m") {
        Some(_) => { common::Metric::PSNR },
        None => { common::Metric::PSNR },
    };

    Ok(common::RPOption {
        ref_path: ref_path,
        dis_path: dis_path,
        width: width,
        height: height,
        metric: metric,
    })
}

fn compute_yuv_psnr(option: &common::RPOption) -> Option<std::io::Error> {
    let mut ref_yuv = YUV::new(option.width, option.height);
    let mut ref_file = File::open(&option.ref_path).unwrap();
    let mut dis_yuv = YUV::new(option.width, option.height);
    let mut dis_file = File::open(&option.dis_path).unwrap();

    loop {
        match ref_yuv.read_frame(&mut ref_file) {
            Ok(0) => { return None },
            Ok(_) => {}
            Err(why) => { return Some(why) }
        };
        match dis_yuv.read_frame(&mut dis_file) {
            Ok(0) => { return None },
            Ok(_) => {},
            Err(why) => { return Some(why) }
        };
        let (y_psnr, u_psnr, v_psnr) = compute_frame_psnr(&ref_yuv, &dis_yuv);
        println!("{:?} {:?} {:?}", y_psnr, u_psnr, v_psnr);
    }
}

fn sse(l: &[u8], r: &[u8]) -> u64 {
    let n = l.len();
    let mut sum_square_error = 0u64;
    for x in 0..n {
        let lv = l[x] as i16;
        let rv = r[x] as i16;
        let error = lv - rv;
        sum_square_error += (error * error) as u64;
    }
    sum_square_error
}

fn compute_psnr(l: &[u8], r: &[u8]) -> f64 {
    let sse_v = sse(l, r) as f64;
    if sse_v < 1_f64 {
        return std::f64::INFINITY;
    }
    let mse = sse_v / l.len() as f64;
    let max = 255.0 * 255.0;
    let val = max / mse;
    10.0 *  val.log10()
}

fn compute_frame_psnr(l: &YUV, r: &YUV) -> (f64, f64, f64) {
    let y_psnr = compute_psnr(l.y.as_slice(), r.y.as_slice());
    let u_psnr = compute_psnr(l.u.as_slice(), r.u.as_slice());
    let v_psnr = compute_psnr(l.v.as_slice(), r.v.as_slice());
    (y_psnr, u_psnr, v_psnr)
}

pub struct YUV {
    y: Vec<u8>,
    u: Vec<u8>,
    v: Vec<u8>,
}

impl YUV {
    fn new(width: u32, height: u32) -> YUV {
        let y_size = (width * height) as usize;
        let uv_size = (y_size >> 2) as usize;
        YUV {
            y: vec![0; y_size],
            u: vec![0; uv_size],
            v: vec![0; uv_size],
        }
    }

    fn read_frame(&mut self, file: &mut File) -> io::Result<i32> {
        let mut reader = BufReader::new(file);
        match reader.read(self.y.as_mut_slice()) {
            Ok(0) => { return Ok(0) },
            Ok(_) => {},
            Err(why) => { return Err(why) }
        }
        reader.read(self.u.as_mut_slice())?;
        reader.read(self.v.as_mut_slice())?;
        Ok(1)
    }
}

fn main() {
    let option = match validate_option() {
        Ok(m) => { m },
        Err(why) => { panic!(why.to_string()) }
    };
    compute_yuv_psnr(&option);
}
