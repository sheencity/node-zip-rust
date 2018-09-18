const addon = require("../native");
const dgram = require("dgram");
const { Observable } = require("rxjs");
const { distinctUntilChanged } = require("rxjs/operators")

function compress(src, filepath) {
    const server = dgram.createSocket("udp4");
    let total;
    let count = 0;
    return new Observable(observer => {
        server
            .on("message", () => observer.next(Math.floor(++count / total * 100) / 100))
            .on("close", () => observer.complete())
            .on("error", err => observer.error(err))
            .bind(() => total = addon.compress(
                server.address().port,
                src,
                filepath,
                () => server.close(),
            ))
    }).pipe(distinctUntilChanged());
}

module.exports = compress;
