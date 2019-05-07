#![cfg(nightly)]
#![feature(test)]

#[macro_use]
extern crate may;
extern crate rand;
extern crate test;

use coroutine::*;
use may::coroutine;
use may::net::{TcpListener, TcpStream};
use may::sync::mpsc;
use rand::Rng;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use test::Bencher;

#[bench]
fn tcp(b: &mut Bencher) {
    may::config().set_workers(4).set_io_workers(4);
    let mut rng = rand::thread_rng();
    let mut socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), rand::thread_rng().gen());
    let mut iter = 0;
    b.iter(move || {
        let listener;
        loop {
            match TcpListener::bind(&socket) {
                Ok(listen) => {
                    listener = listen;
                    break;
                }
                Err(_) => socket.set_port(rng.gen()),
            }
        }
        let s2 = socket.clone();
        let (sender, receiver) = mpsc::channel();
        join!(
            go!(move || {
                for (i, stream) in listener.incoming().enumerate() {
                    let mut stream = stream.expect("bad stream");
                    go!(move || stream.read_exact(&mut [0u8]).expect("oy"));
                }
            }),
            go!(move || {
                for i in 0..1000 {
                    let mut stream = TcpStream::connect(s2).expect("oof");
                    go!(move || stream.write_all(&[0u8]).expect("failed to write"));
                }
                println!("handled streams: {:?}", iter);
                sender.send(0u8).expect("failed to send");
            })
        );

        iter += 1;
        assert_eq!(receiver.recv().expect("failed to receive"), 0)
    })
}

#[bench]
fn yield_bench(b: &mut Bencher) {
    // don't print any panic info
    // when cancel the generator
    // panic::set_hook(Box::new(|_| {}));
    b.iter(|| {
        let h = go!(|| for _i in 0..10000 {
            yield_now();
        });

        h.join().unwrap();
    });
}

#[bench]
fn spawn_bench(b: &mut Bencher) {
    b.iter(|| {
        let total_work = 1000;
        let threads = 2;
        let mut vec = Vec::with_capacity(threads);
        for _t in 0..threads {
            let j = std::thread::spawn(move || {
                scope(|scope| {
                    for _i in 0..total_work / threads {
                        go!(scope, || {
                            // yield_now();
                        });
                    }
                });
            });
            vec.push(j);
        }
        for j in vec {
            j.join().unwrap();
        }
    });
}

#[bench]
fn spawn_bench_1(b: &mut Bencher) {
    may::config().set_pool_capacity(10000);
    b.iter(|| {
        let total_work = 1000;
        let threads = 2;
        let mut vec = Vec::with_capacity(threads);
        for _t in 0..threads {
            let work = total_work / threads;
            let j = std::thread::spawn(move || {
                let v = (0..work).map(|_| go!(|| {})).collect::<Vec<_>>();
                for h in v {
                    h.join().unwrap();
                }
            });
            vec.push(j);
        }
        for j in vec {
            j.join().unwrap();
        }
    });
}

#[bench]
fn smoke_bench(b: &mut Bencher) {
    may::config().set_pool_capacity(10000);
    b.iter(|| {
        let threads = 5;
        let mut vec = Vec::with_capacity(threads);
        for _t in 0..threads {
            let j = std::thread::spawn(|| {
                scope(|scope| {
                    for _i in 0..200 {
                        go!(scope, || for _j in 0..1000 {
                            yield_now();
                        });
                    }
                });
            });
            vec.push(j);
        }
        for j in vec {
            j.join().unwrap();
        }
    });
}

#[bench]
fn smoke_bench_1(b: &mut Bencher) {
    may::config().set_pool_capacity(10000);
    b.iter(|| {
        let threads = 5;
        let mut vec = Vec::with_capacity(threads);
        for _t in 0..threads {
            let j = std::thread::spawn(|| {
                scope(|scope| {
                    for _i in 0..2000 {
                        go!(scope, || for _j in 0..4 {
                            yield_now();
                        });
                    }
                });
            });
            vec.push(j);
        }
        for j in vec {
            j.join().unwrap();
        }
    });
}

#[bench]
fn smoke_bench_2(b: &mut Bencher) {
    may::config().set_pool_capacity(10000);
    b.iter(|| {
        scope(|s| {
            // create a main coroutine, let it spawn 10 sub coroutine
            for _ in 0..100 {
                go!(s, || {
                    scope(|ss| {
                        for _ in 0..100 {
                            go!(ss, || {
                                // each task yield 4 times
                                for _ in 0..4 {
                                    yield_now();
                                }
                            });
                        }
                    });
                });
            }
        });
    });
}

#[bench]
fn smoke_bench_3(b: &mut Bencher) {
    b.iter(|| {
        let mut vec = Vec::with_capacity(100);
        // create a main coroutine, let it spawn 10 sub coroutine
        for _ in 0..100 {
            let j = go!(|| {
                let mut _vec = Vec::with_capacity(100);
                for _ in 0..100 {
                    let _j = go!(|| {
                        // each task yield 10 times
                        for _ in 0..4 {
                            yield_now();
                        }
                    });
                    _vec.push(_j);
                }
                for _j in _vec {
                    _j.join().ok();
                }
            });
            vec.push(j);
        }
        for j in vec {
            j.join().ok();
        }
    });
}
