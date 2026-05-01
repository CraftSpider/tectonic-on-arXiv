import {queue} from 'async'
import {get_merge_base, run_check} from './run.js'
import {Job} from './misc.js'

declare global {
    namespace NodeJS {
        interface ProcessEnv {
            GITHUB_SHA: string;
            GITHUB_HEAD_REF: string;
            GITHUB_BASE_REF?: string;
            GITHUB_RUN_ID: string;
        }
    }
}

const jobs = queue<Job>(async (job, _) => await run_check(job), 1)

async function main() {
    let head_sha: string = process.env.GITHUB_SHA
    let head_branch: string = process.env.GITHUB_HEAD_REF
    let base_sha: string | undefined = process.env.GITHUB_BASE_REF
    let check_run_id: number = parseInt(process.env.GITHUB_RUN_ID);
    if (!base_sha) {
        throw new Error('GITHUB_BASE_REF is not set')
    }
    base_sha = await get_merge_base(head_sha, base_sha)
    console.log(`queueing run: base=${base_sha} head=${head_sha}`)
    // ensure that base_sha has a report ready
    // jobs.push({head_sha: base_sha})
    // start regression check
    jobs.push({head_sha, head_branch, base_sha, check_run_id})
}

await main();
