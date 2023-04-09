pub struct Candidate {
    pub fullname: String,
    pub linkedin: String,
    pub message: String,
}

impl Candidate {
    pub fn new(fullname: String, linkedin: String, message: String) -> Self {
        Candidate {
            fullname,
            linkedin,
            message,
        }
    }
}
