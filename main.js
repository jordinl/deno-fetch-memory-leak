import {Sema as Semaphore} from "npm:async-sema";

const file = await Deno.readTextFile("urls.txt");
const urls = file.split("\n");
const CONCURRENCY = parseInt(Deno.env.get('CONCURRENCY') || 10);
const LIMIT = parseInt(Deno.env.get('LIMIT') || urls.length);
let results = {
    '2XX': 0,
    '3XX': 0,
    '4XX': 0,
    '5XX': 0,
    'error': 0
};
let processed = 0;

const log = (...messages) => console.log(`[${new Date().toISOString()}]`, ...messages)

const printMemory = () => {
    const { rss } = Deno.memoryUsage();
    const exp = rss === 0 ? 0 : Math.floor(Math.log(rss) / Math.log(1024));
    const memory = +((rss / Math.pow(1024, exp)).toFixed(2)) + ' ' + ['B', 'kB', 'MB', 'GB', 'TB'][exp];
    log(`Memory usage: ${memory} -- Processed: ${processed}`);
}

const client = Deno.createHttpClient({poolMaxIdlePerHost: 0});

const makeRequest = async url => {
    const signal = AbortSignal.timeout(5000);
    try {
        const response = await fetch(url, { signal, client });
        await response.text();
        results[`${response.status.toString()[0]}XX`] += 1;
    } catch (error) {
        results.error += 1;
    }
    processed += 1;
}

const semaphore = new Semaphore(CONCURRENCY);

log(`Using CONCURRENCY: ${CONCURRENCY}`);
log(`Using LIMIT: ${LIMIT}`);

printMemory();
const interval = setInterval(printMemory, 10000);
const time = Date.now();

for (const url of urls.slice(0, LIMIT)) {
    await semaphore.acquire();
    (async () => {
        await makeRequest(url);
        semaphore.release();
    })()
}

await semaphore.drain();
clearInterval(interval);
printMemory();

log('results: ', results);
log(`Total time: ${(Date.now() - time) / 1000}s`);
