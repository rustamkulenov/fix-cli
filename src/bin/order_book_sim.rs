use std::io::{self, Write};
use std::{cmp, fmt};

use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

const MAX_LEVELS: usize = 10;
const MIN_PRICE_STEP: u32 = 1;
const MAX_PRICE_STEP: u32 = 10;
const MIN_SIZE: u32 = 1;
const MAX_SIZE: u32 = 1000;
/// BeginString(8) with BodyLength(9) fields without value. Shall be followed by body length.
const MSG_HEADER_PART_1: &'static [u8] = "8=FIX.4.4\u{01}9=".as_bytes();
// MsgType(35), SenderCompId(49), MsgSeqNum(34), TargetCompId(56), SendingTime(52) fields with values.
const MSG_HEADER_PART_2: &'static [u8] =
    "35=W\u{01}49=SENDER_ID\u{01}56=TARGET_ID\u{01}34=12345\u{01}52=YYYYMMDD-HH:MM:SS.sss\u{01}"
        .as_bytes();
const MSG_TRAILER: &'static [u8] = "10=543\u{01}".as_bytes();

const ZERO: u8 = 0x30;

/// Price, size information for an order book level.
#[derive(Clone, Copy, Debug)]
struct PxSz {
    pub price: u32,
    pub size: u32,
}

impl PxSz {
    pub fn new() -> PxSz {
        PxSz { price: 0, size: 0 }
    }
}

/// On-stack implementation of an OrderBook.
/// Elements of bids and asks are sorted from the spread.
struct OrderBook {
    pub asks: [PxSz; MAX_LEVELS],
    pub bids: [PxSz; MAX_LEVELS],
    pub ask_num: usize,
    pub bid_num: usize,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: [PxSz::new(); MAX_LEVELS],
            bids: [PxSz::new(); MAX_LEVELS],
            ask_num: 0,
            bid_num: 0,
        }
    }

    /// Generates an orderbook.
    /// Prices are Uniformly distributed around mid price.
    /// TODO: size is Normaly distributed around a mid price.
    pub fn generate(&mut self, bid_num: usize, ask_num: usize, mid_price: u32) -> () {
        self.ask_num = ask_num;
        self.bid_num = bid_num;

        let mut rng = thread_rng();
        let price_step_between = Uniform::from(MIN_PRICE_STEP..MAX_PRICE_STEP); // generates in a range [min..max)
        let size_between = Uniform::from(MIN_SIZE..MAX_SIZE); // generates in a range [min..max)

        let mut price: u32 = mid_price;
        for i in 0..bid_num - 1 {
            let bid = &mut self.bids[i];
            price -= price_step_between.sample(&mut rng);

            bid.price = price;
            bid.size = size_between.sample(&mut rng);
        }

        let mut price: u32 = mid_price;
        for i in 0..ask_num - 1 {
            let ask = &mut self.asks[i];
            price += price_step_between.sample(&mut rng);

            ask.price = price;
            ask.size = size_between.sample(&mut rng);
        }
    }
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BID\t\t\t\tASK")?;
        writeln!(f, "px\tsz\t\t\tpx\tsz")?;
        writeln!(f, "-----\t-----\t\t\t-----\t-----")?;
        for i in 0..cmp::max(self.ask_num, self.bid_num) - 1 {
            let bid = self.bids[i];
            let ask = self.asks[i];
            writeln!(
                f,
                "{}\t{}\t\t\t{}\t{}",
                bid.price, bid.size, ask.price, ask.size
            )?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct AsciiWriter<'a> {
    /// A buffer to write into
    pub buf: &'a mut [u8],
    idx: usize,
}

impl AsciiWriter<'_> {
    pub fn new(buf: &mut [u8]) -> AsciiWriter {
        AsciiWriter { buf, idx: 0 }
    }

    pub fn len(&self) -> usize {
        self.idx
    }

    /// Drops idx pointer to zero.
    pub fn clear(&mut self) -> () {
        self.idx = 0;
    }

    /// Writes a byte as it is into the buffer.
    pub fn write_raw_u8(&mut self, v: u8) -> () {
        self.buf[self.idx] = v;
        self.idx += 1;
    }

    /// Converts u32 value in ASCII representation and writes it to the buffer.
    /// No memory is allocated. Complexity: O(n+n/2), where n - number of symbols in the provided value (i.e. max power of a base).
    /// See also:
    ///     https://github.com/miloyip/itoa-benchmark
    ///     https://docs.rs/itoap/latest/itoap/
    pub fn write_u32(&mut self, value: u32) -> () {
        const BASE: u32 = 10;
        let mut v = value;
        let mut i: usize = self.idx;

        // Write ASCII
        loop {
            let number = v % BASE;
            self.buf[self.idx] = ZERO + number as u8;
            self.idx += 1;
            v = v / BASE;
            if v == 0 {
                break;
            };
        }
        // Revert symbols
        let mut j: usize = self.idx - 1;
        loop {
            if i >= j {
                break;
            }
            let tmp = self.buf[i];
            self.buf[i] = self.buf[j];
            self.buf[j] = tmp;
            i += 1;
            j -= 1;
        }
    }

    pub fn write_usize(&mut self, value: usize) -> () {
        const BASE: usize = 10;
        let mut v = value;
        let mut i: usize = self.idx;

        // Write ASCII
        loop {
            let number = v % BASE;
            self.buf[self.idx] = ZERO + number as u8;
            self.idx += 1;
            v = v / BASE;
            if v == 0 {
                break;
            };
        }
        // Revert symbols
        let mut j: usize = self.idx - 1;
        loop {
            if i >= j {
                break;
            }
            let tmp = self.buf[i];
            self.buf[i] = self.buf[j];
            self.buf[j] = tmp;
            i += 1;
            j -= 1;
        }
    }

    /// Writes an array of bytes.
    pub fn write_buf(&mut self, value: &[u8]) -> () {
        let len = value.len();
        self.buf[self.idx..self.idx + len].copy_from_slice(value);
        self.idx += len;
    }
}

fn write_orderbook(w: &mut AsciiWriter, dom: &OrderBook) {
    // NoMdEntries(268)
    w.write_buf(b"268=");
    w.write_usize(dom.ask_num + dom.bid_num);
    w.write_raw_u8(1);
    // TODO: create a function to write tag, value, SOH without tag conversion to bytes.

    for i in 0..dom.bid_num {
        let bid = dom.bids[i];
        w.write_buf(b"269=0"); // MDEntryType(268)
        w.write_raw_u8(1);
        w.write_buf(b"270="); // MDEntryPx(270)
        w.write_u32(bid.price);
        w.write_raw_u8(1);
        w.write_buf(b"271="); // MDEntrySize(271)
        w.write_u32(bid.size);
        w.write_raw_u8(1);
    }

    for i in 0..dom.ask_num {
        let ask = dom.asks[i];
        w.write_buf(b"269=1"); // MDEntryType(268)
        w.write_raw_u8(1);
        w.write_buf(b"270="); // MDEntryPx(270)
        w.write_u32(ask.price);
        w.write_raw_u8(1);
        w.write_buf(b"271="); // MDEntrySize(271)
        w.write_u32(ask.size);
        w.write_raw_u8(1);
    }
}

fn main() -> std::io::Result<()> {
    let mut dom = OrderBook::new();
    let mut header_buf: [u8; 255] = [0u8; 255];
    let mut body_buf: [u8; 1024] = [0u8; 1024];

    let mut header_aw = AsciiWriter::new(&mut header_buf);
    let mut body_aw = AsciiWriter::new(&mut body_buf);

    let mut stdout = io::stdout().lock();

    for _ in 0..10_000_000 {
        header_aw.clear();
        body_aw.clear();
        dom.generate(10, 10, 1000);
        //println!("{}", dom);

        header_aw.write_buf(MSG_HEADER_PART_1);
        body_aw.write_buf(MSG_HEADER_PART_2);
        body_aw.write_buf("55=EURUSD\u{01}".as_bytes());
        write_orderbook(&mut body_aw, &dom);

        header_aw.write_usize(body_aw.len());
        header_aw.write_raw_u8(1);
        body_aw.write_buf(MSG_TRAILER);

        //println!("{0:?}", header_aw);
        //println!("{0:?}", body_aw);

        stdout.write_all(&header_aw.buf[..header_aw.len()])?;
        stdout.write_all(&body_aw.buf[..body_aw.len()])?;
    }

    Ok(())
}
