use criterion::{black_box, criterion_group, criterion_main, Criterion};
use electricity_monitor::infrastructure::UrlBuilder;

fn bench_fastprefix_with_roomid_u32(c: &mut Criterion) {
    // 固定前缀: 末尾是 `?roomid=`，没有后续 `&`，走 FastPrefix
    let builder =
        UrlBuilder::from_template("https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=")
            .unwrap();

    c.bench_function("fastprefix_with_roomid_u32", |b| {
        b.iter(|| {
            let id = black_box(3243u32);
            let url = builder.with_roomid_u32(id);
            black_box(url);
        })
    });
}

fn bench_generic_with_roomid(c: &mut Criterion) {
    // 泛型路径: `roomid` 后还有 `&b=2`，将回退到 Generic
    let builder = UrlBuilder::from_template("https://example.com?a=1&roomid=3243&b=2").unwrap();

    c.bench_function("generic_with_roomid", |b| {
        b.iter(|| {
            let s = black_box("98765");
            let url = builder.with_roomid(s);
            black_box(url);
        })
    });
}

criterion_group!(
    benches,
    bench_fastprefix_with_roomid_u32,
    bench_generic_with_roomid
);
criterion_main!(benches);
