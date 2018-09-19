const addon = require("../native");
const dgram = require("dgram");
const { Observable } = require("rxjs");
const { distinctUntilChanged } = require("rxjs/operators")

const compress = (src, filepath) => new Observable(observer => {
    let total, count = 0;
    const server = dgram.createSocket("udp4"); server
        .on("message", () => observer.next(Math.floor(++count / total * 100) / 100))
        .on("close", () => observer.complete())
        .on("error", err => observer.error(err))
        .bind(() => total = addon.compress(
            server.address().port,
            src,
            filepath,
            () => server.close(),
        ))
}).pipe(distinctUntilChanged())

module.exports = compress;
