// Copyright (c) 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::future::Future;

#[trait_variant::make(IntFactory: Send)]
pub trait LocalIntFactory {
    const NAME: &'static str;

    type MyFut<'a>: Future
    where
        Self: 'a;

    async fn make(&self, x: u32, y: &str) -> i32;
    fn stream(&self) -> impl Iterator<Item = i32>;
    fn call(&self) -> u32;
    fn another_async(&self, input: Result<(), &str>) -> Self::MyFut<'_>;
    async fn defaulted(&self) -> i32 {
        0
    }
}

#[allow(dead_code)]
fn spawn_task(factory: impl IntFactory + 'static) {
    tokio::spawn(async move {
        let _int = factory.make(1, "foo").await;
    });
}

fn main() {}
