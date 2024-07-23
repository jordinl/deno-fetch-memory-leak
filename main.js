import {Sema as Semaphore} from "npm:async-sema";

const CONCURRENCY = parseInt(Deno.env.get('CONCURRENCY') || 10)

const file = await Deno.readTextFile("urls.txt");
const urls = file.split("\n");

const makeRequest = async url => {
    const signal = AbortSignal.timeout(5000);
    let code;
    try {
        const response = await fetch(url, { signal });
        await response.text();
        code = response.status;
    } catch (error) {
        code = error;
    }
    const { rss } = Deno.memoryUsage();
    console.log(`mem: ${rss} (${url} | ${code})`);
}

const semaphore = new Semaphore(CONCURRENCY);

console.log(`Using CONCURRENCY: ${CONCURRENCY}`)

for (const url of urls) {
    await semaphore.acquire();
    (async () => {
        await makeRequest(url);
        semaphore.release();
    })()
}

semaphore.drain();
