import {get_merge_base, run_check} from './run.js'

declare global {
    namespace NodeJS {
        interface ProcessEnv {
            GITHUB_SHA: string;
            GITHUB_HEAD_REF: string;
            GITHUB_BASE_REF?: string;
            GITHUB_RUN_ID: string;
            TECTONIC_WORKSPACE?: string;
            HEAD_COMMIT: string;
            BASE_COMMIT?: string;
        }
    }
}

async function main() {
    let head_sha: string = process.env.HEAD_COMMIT;
    let head_branch: string = process.env.GITHUB_HEAD_REF;
    let base_sha1: string | undefined = process.env.BASE_COMMIT;
    let check_run_id: number = parseInt(process.env.GITHUB_RUN_ID);
    // ensure that base_sha has a report ready
    // jobs.push({head_sha: base_sha})
    // start regression check
    if (base_sha1) {
        console.log(`queueing run: base=${base_sha1} head=${head_sha}`)
        let base_sha = await get_merge_base(head_sha, base_sha1)
        await run_check({head_sha, head_branch, base_sha, check_run_id})
    } else {
        console.log(`queueing run: head=${head_sha}`)
        await run_check({head_sha, check_run_id})
    }
}

await main();
