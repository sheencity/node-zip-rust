const addon = require("../native");
const dgram = require("dgram");
const { Observable } = require("rxjs");
const { distinctUntilChanged } = require("rxjs/operators");

const extract = (zipfile, dest) => new Observable(observer => {
    let total, count = 0;
    const server = dgram.createSocket("udp4"); server
        .on("message", (msg, rinfo) => observer.next(Math.floor(++count / total * 100) / 100))
        .on("close", () => observer.complete())
        .on("error", err => observer.error(err))
        .bind(() => total = addon.extract(
            server.address().port,
            zipfile,
            dest,
            () => (server.close(), observer.complete()),
        ))
}).pipe(distinctUntilChanged())

module.exports = extract;
