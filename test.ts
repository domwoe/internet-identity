import { remote } from "webdriverio";
import { spawn } from "child_process";
import { get } from "http";

// @ts-ignore
const log  = require("why-is-node-running");


console.log("hello");


function isAccessible(opts: { hostname: string, port: number} ): Promise<boolean> {
    return new Promise((resolve) => {
        get(opts, (response) => {
            resolve(true); // TODO: note that care only if replied
        }).on('error', () => {
            resolve(false);
        })
    });
}

function waitASec(): Promise<void> {
    return new Promise((resolve) => {
        setTimeout(resolve, 1000);
    });
}

async function waitUntilAccessible(opts: { hostname: string, port: number}, retries: number) {
    for (let i = 0; i < retries; i++) {

        console.log(i);

        const ready = await isAccessible(opts);

        if (ready) {
            return;
        }

        await waitASec();
    }

    console.log("Could not access");
    throw Error('bwah');

}

async function shoot() {
    const showcase = spawn("npm run showcase", { shell: true });

    showcase.on('close', (code, signal) => {
        console.log(`showcase was closed (${code}, ${signal})`);
    });

    showcase.on('exit', (code, signal) => {
        console.log(`showcase exited (${code}, ${signal})`);
    });

    showcase.on('error', (e) => {
        console.log(`showcase errored (${e})`);
    });

    try {
        await waitUntilAccessible({ hostname: "localhost", port: 8080}, 10);

        await screenshotMeThis({ url: "http://localhost:8080/authenticate" });
        console.log("ok we're good");

    } catch (e) {
        console.log(e);

    } finally {
        console.log("killing showcase");
        console.log(showcase.kill());
        console.log("ok now what");
    }

// @ts-ignore
    //log();
}

shoot();


async function screenshotMeThis(opts: { url: string }) {
    const browser = await remote({
        capabilities: {
            browserName: "chrome",
            "goog:chromeOptions": {
                args: ["headless", "disable-gpu"],
            },
        },
    })

    await browser.url(opts.url)

    browser.waitUntil(
        () => browser.execute(() => document.readyState === 'complete'),
            {
            timeout: 10 * 1000,
            timeoutMsg: 'Browser did not load after 10 seconds'
        }
    );

    await browser.saveScreenshot('./screenshots/my-new-screenshot.png');
    await browser.deleteSession();
    await browser.pause(1);
}

