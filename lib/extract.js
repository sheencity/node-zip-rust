const addon = require("../native");
const dgram = require("dgram");
const { Observable } = require("rxjs");
const { distinctUntilChanged } = require("rxjs/operators");

function extract(zipfile, dest) {
    const server = dgram.createSocket("udp4");
    let total;
    let count = 0;
    return new Observable(observer => {
        server
            .on("message", (msg, rinfo) => observer.next(Math.floor(++count / total * 100) / 100))
            .on("close", () => observer.complete())
            .on("error", err => observer.error(err))
            .bind(() => total = addon.extract(
                server.address().port,
                zipfile,
                dest,
                () => (server.close(), observer.complete()),
            ))
    }).pipe(distinctUntilChanged());

}

module.exports = extract;
