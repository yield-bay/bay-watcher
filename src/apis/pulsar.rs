use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use struct_iterable::Iterable;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub code: i64,
    pub errors: Value,
    pub message: String,
    pub result: Result,
    pub is_success: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub farm_pools: HashMap<String, FarmPool>,
    pub farms: HashMap<String, String>,
    pub updated_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmPool {
    pub farmind_id: String,
    pub last_apr: String,
    pub reward_token_apr: String,
    pub reward_token: String,
    pub bonus_token_apr: String,
    pub bonus_token: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct FarmPools {
//     #[serde(rename = "0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5")]
//     pub n0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5: n0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Farms {
//     #[serde(rename = "0x09e2f5ff5e7e9014fdcb6236dc4a6ee44175e1e67dd4f1a353bdcf90ffbb0934")]
//     pub n0x09e2f5ff5e7e9014fdcb6236dc4a6ee44175e1e67dd4f1a353bdcf90ffbb0934: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Iterable)]
#[serde(rename_all = "camelCase")]
pub struct PoolsAPRRoot {
    pub code: i64,
    pub errors: Value,
    pub message: String,
    pub result: PoolsAPRResult,
    pub is_success: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Iterable)]
#[serde(rename_all = "camelCase")]
pub struct PoolsAPRResult {
    #[serde(rename = "0x05148fcd032e6a04b8067a4b506b8338435b74fe")]
    pub n0x05148fcd032e6a04b8067a4b506b8338435b74fe: f64,
    #[serde(rename = "0x06159391efcb8d050506f4fd787a191f7598540a")]
    pub n0x06159391efcb8d050506f4fd787a191f7598540a: f64,
    #[serde(rename = "0x0f838b767d1fdda0173f2e2b38d9d8befa970271")]
    pub n0x0f838b767d1fdda0173f2e2b38d9d8befa970271: f64,
    #[serde(rename = "0x0fc66f6592cb5589311eadde716380ec7e5adb00")]
    pub n0x0fc66f6592cb5589311eadde716380ec7e5adb00: f64,
    #[serde(rename = "0x1103258b909ad741df79c2c91aa133852eb0003d")]
    pub n0x1103258b909ad741df79c2c91aa133852eb0003d: f64,
    #[serde(rename = "0x152bddf0150a9a4e5b717b01ce989b5147e38403")]
    pub n0x152bddf0150a9a4e5b717b01ce989b5147e38403: f64,
    #[serde(rename = "0x1b11d991f32fb59ec4ee744de68ad65d9e85b2d2")]
    pub n0x1b11d991f32fb59ec4ee744de68ad65d9e85b2d2: f64,
    #[serde(rename = "0x2f7daf9b66a7bee6f9e046973e2e3d01d810207c")]
    pub n0x2f7daf9b66a7bee6f9e046973e2e3d01d810207c: f64,
    #[serde(rename = "0x34983cd4448562d9dd2b67ed687cff16a5a939b1")]
    pub n0x34983cd4448562d9dd2b67ed687cff16a5a939b1: f64,
    #[serde(rename = "0x3909a2e64f27c257f1189cb628e164b696acb49c")]
    pub n0x3909a2e64f27c257f1189cb628e164b696acb49c: f64,
    #[serde(rename = "0x3ceb7d674236adbfe9fa1ada6c48e4a92e6441b4")]
    pub n0x3ceb7d674236adbfe9fa1ada6c48e4a92e6441b4: f64,
    #[serde(rename = "0x416bd9798d5214cae6f837c0a53a73beb3ced465")]
    pub n0x416bd9798d5214cae6f837c0a53a73beb3ced465: f64,
    #[serde(rename = "0x41fcc3a91e2063e70dbe86311bd30252ea1bb26f")]
    pub n0x41fcc3a91e2063e70dbe86311bd30252ea1bb26f: f64,
    #[serde(rename = "0x434714e057ed2741f4af15e3ea62c787a4fb5eef")]
    pub n0x434714e057ed2741f4af15e3ea62c787a4fb5eef: f64,
    #[serde(rename = "0x4977dff8adadcb6b35b9af3b5548564ae69c6d41")]
    pub n0x4977dff8adadcb6b35b9af3b5548564ae69c6d41: f64,
    #[serde(rename = "0x4d2cad7164fd38f3d6ee225c98727a641801cff3")]
    pub n0x4d2cad7164fd38f3d6ee225c98727a641801cff3: f64,
    #[serde(rename = "0x4df03860c15f90205092f026acbadc685ff91af9")]
    pub n0x4df03860c15f90205092f026acbadc685ff91af9: f64,
    #[serde(rename = "0x4e855d0fde22ce095071469f6bb5ca4ca508bba6")]
    pub n0x4e855d0fde22ce095071469f6bb5ca4ca508bba6: f64,
    #[serde(rename = "0x5daf7f80cc550ee6249a4635c3bb0678e94d3867")]
    pub n0x5daf7f80cc550ee6249a4635c3bb0678e94d3867: f64,
    #[serde(rename = "0x61b600a9f0e4b53cf3f6a837c2815964e2291f9c")]
    pub n0x61b600a9f0e4b53cf3f6a837c2815964e2291f9c: f64,
    #[serde(rename = "0x70419f443b58cc979ae472e571709a55455b2f53")]
    pub n0x70419f443b58cc979ae472e571709a55455b2f53: f64,
    #[serde(rename = "0x79eb71c1592a678c234ea221ed3fdc10cee89f88")]
    pub n0x79eb71c1592a678c234ea221ed3fdc10cee89f88: f64,
    #[serde(rename = "0x79fb677df79108c83f797a3a196fbccb1dc28832")]
    pub n0x79fb677df79108c83f797a3a196fbccb1dc28832: f64,
    #[serde(rename = "0x7cbe7174fbe857c0097dbaa1fdfde627e3eba4f7")]
    pub n0x7cbe7174fbe857c0097dbaa1fdfde627e3eba4f7: f64,
    #[serde(rename = "0x7e71d586ad01c0bf7953eb82e7b76c1338b0068c")]
    pub n0x7e71d586ad01c0bf7953eb82e7b76c1338b0068c: f64,
    #[serde(rename = "0x7fd757059e032977f79d96765dac18f0d687538f")]
    pub n0x7fd757059e032977f79d96765dac18f0d687538f: f64,
    #[serde(rename = "0x80147017c469b97da3eefc10679c81d1a9d99b66")]
    pub n0x80147017c469b97da3eefc10679c81d1a9d99b66: f64,
    #[serde(rename = "0x813d21f1270edf98f79508fddf4e1395d51e1537")]
    pub n0x813d21f1270edf98f79508fddf4e1395d51e1537: f64,
    #[serde(rename = "0x86c50c9bc4e3f15d6d6eb47393cb07bd701bcf04")]
    pub n0x86c50c9bc4e3f15d6d6eb47393cb07bd701bcf04: f64,
    #[serde(rename = "0x8a2bc44019daf52add9bddb3e73c41e4b84cd449")]
    pub n0x8a2bc44019daf52add9bddb3e73c41e4b84cd449: f64,
    #[serde(rename = "0x8acda80501a2ac85173393efeb05ad20ef40314c")]
    pub n0x8acda80501a2ac85173393efeb05ad20ef40314c: f64,
    #[serde(rename = "0x909648c41d2bd216ac4a380c5bc7ea32e5cedda8")]
    pub n0x909648c41d2bd216ac4a380c5bc7ea32e5cedda8: f64,
    #[serde(rename = "0x9139de5f21b83f6db9f0246b4a4a74310b1aa1cb")]
    pub n0x9139de5f21b83f6db9f0246b4a4a74310b1aa1cb: f64,
    #[serde(rename = "0x93d2b0a9dfa6510f4bd3c5a03ca9be26dbcae431")]
    pub n0x93d2b0a9dfa6510f4bd3c5a03ca9be26dbcae431: f64,
    #[serde(rename = "0x99935dec67026d0dd08410329a3f7a3cae4370fd")]
    pub n0x99935dec67026d0dd08410329a3f7a3cae4370fd: f64,
    #[serde(rename = "0xaa3cc4653abc161397f65fc8435ced0fff55657a")]
    pub n0xaa3cc4653abc161397f65fc8435ced0fff55657a: f64,
    #[serde(rename = "0xab8c35164a8e3ef302d18da953923ea31f0fe393")]
    pub n0xab8c35164a8e3ef302d18da953923ea31f0fe393: f64,
    #[serde(rename = "0xac8daa0d4a17f7ec4452ac71fbfe9569dbab52e8")]
    pub n0xac8daa0d4a17f7ec4452ac71fbfe9569dbab52e8: f64,
    #[serde(rename = "0xade92ae9c6235a9081b71b6a488ffebe7da037ef")]
    pub n0xade92ae9c6235a9081b71b6a488ffebe7da037ef: f64,
    #[serde(rename = "0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5")]
    pub n0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5: f64,
    #[serde(rename = "0xb6cd370fb6fdd6d624a4e782e836d8ac173d0f92")]
    pub n0xb6cd370fb6fdd6d624a4e782e836d8ac173d0f92: f64,
    #[serde(rename = "0xb8a088f10f1fe0631582ef5a536d2d717f4ea4d7")]
    pub n0xb8a088f10f1fe0631582ef5a536d2d717f4ea4d7: f64,
    #[serde(rename = "0xc3457eafed83b76d57cedfb89db18610c0b3ba3d")]
    pub n0xc3457eafed83b76d57cedfb89db18610c0b3ba3d: f64,
    #[serde(rename = "0xc79b80c300ec6b14deba1cb6693b18314d41d9f7")]
    pub n0xc79b80c300ec6b14deba1cb6693b18314d41d9f7: f64,
    #[serde(rename = "0xd14efe86b66124b322b0b7e0ec4ca22d9201d8cd")]
    pub n0xd14efe86b66124b322b0b7e0ec4ca22d9201d8cd: f64,
    #[serde(rename = "0xd3c255da9134aad0689275bd81de5d532e6f7071")]
    pub n0xd3c255da9134aad0689275bd81de5d532e6f7071: f64,
    #[serde(rename = "0xd67425e31982426a74f5579e886446b2a7efa466")]
    pub n0xd67425e31982426a74f5579e886446b2a7efa466: f64,
    #[serde(rename = "0xd72b0608ab56a926d549ab8183a4061f0ef80c52")]
    pub n0xd72b0608ab56a926d549ab8183a4061f0ef80c52: f64,
    #[serde(rename = "0xda6381c6c5a20bc03d0dac2bf9d165d5775f41f7")]
    pub n0xda6381c6c5a20bc03d0dac2bf9d165d5775f41f7: f64,
    #[serde(rename = "0xdd228d01e11041050be93fcfaf3005930782810e")]
    pub n0xdd228d01e11041050be93fcfaf3005930782810e: f64,
    #[serde(rename = "0xde419feec1d1365f11a032edca62ba96634519cd")]
    pub n0xde419feec1d1365f11a032edca62ba96634519cd: f64,
    #[serde(rename = "0xdff059a0df13b06fd9cd92caecd7522285f15ac9")]
    pub n0xdff059a0df13b06fd9cd92caecd7522285f15ac9: f64,
    #[serde(rename = "0xe66ec3a7d22a0c3f4807ebc39ae09d1d6087d9b0")]
    pub n0xe66ec3a7d22a0c3f4807ebc39ae09d1d6087d9b0: f64,
    #[serde(rename = "0xe72de1bdf5bc660799625917c68ff48731ea3c9b")]
    pub n0xe72de1bdf5bc660799625917c68ff48731ea3c9b: f64,
    #[serde(rename = "0xf423a6e47a98521e96f862fd019310b04edb8a05")]
    pub n0xf423a6e47a98521e96f862fd019310b04edb8a05: f64,
    #[serde(rename = "0xf695afac08a35289aaf1ff77c4c0f3ed011e271a")]
    pub n0xf695afac08a35289aaf1ff77c4c0f3ed011e271a: f64,
    // pub updated_at: String,
}

// pub impl PoolsAPRResult {
//     pub fn as_hash_map(&self) -> HashMap<String, i64> {
//         HashMap::clone(&self)
//     }
// }
