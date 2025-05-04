use std::collections::HashSet;
use std::hash::Hash;

pub mod auth;
pub mod jwt;

/// 通用工具类
pub struct Utils;

impl Utils {
    /// 三维向量转一维向量
    pub async fn dedup<T, F, R>(vec_vec: Vec<Vec<T>>, f: F) -> Vec<R>
    where
        F: Fn(&&T) -> R,
        R: Hash + Eq + Clone,
    {
        let mut seen = HashSet::new();
        vec_vec.iter().for_each(|vec| {
            let _ = vec.iter().filter(|v| seen.insert(f(v))).collect::<Vec<_>>();
        });
        seen.into_iter().collect()
    }

}
