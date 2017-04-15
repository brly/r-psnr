pub enum Metric {
    PSNR,
    SSIM,
}

pub struct RPOption {
    pub ref_path: String,
    pub dis_path: String,
    pub width: u32,
    pub height: u32,
    pub metric: Metric,
}
