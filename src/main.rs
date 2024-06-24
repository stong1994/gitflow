use gitflow::status::Status;

fn main() {
    let mut status = Status::Begin;
    loop {
        status = status.call()
    }
}
