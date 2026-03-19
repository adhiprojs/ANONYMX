use zeroize::Zeroize;

pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(text: &str) -> Self {
        Self { data: text.as_bytes().to_vec() }
    }
    pub fn update(&mut self, text: &str) {
        self.data.zeroize();
        self.data = text.as_bytes().to_vec();
    }
    pub fn read(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }
    pub fn zeroize(&mut self) {
        self.data.zeroize();
        self.data.clear();
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
