use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref PRISMA_CLIENT: Mutex<PrismaClient> = Mutex::new(PrismaClient::new());
}

pub fn get_prisma_client() -> &'static PrismaClient {
    PRISMA_CLIENT.lock().unwrap()
}