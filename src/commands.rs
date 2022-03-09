use twilight_interactions::command::ApplicationCommandData;

pub mod basic;
pub mod iqdb;
pub mod saucenao;

pub fn get() -> Vec<ApplicationCommandData> {
    let mut res = Vec::new();
    let mut basic = basic::get();
    let mut iqdb = iqdb::get();
    let mut saucenao = saucenao::get();

    res.append(&mut basic);
    res.append(&mut iqdb);
    res.append(&mut saucenao);

    res
}
