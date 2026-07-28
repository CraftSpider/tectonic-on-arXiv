import {readFileSync, statSync} from "fs";

export interface SampleRun {
    sample: string,
    statuscode: number,
    seconds: number,
    results: { [key: string]: string }
}

export function get_samples(sha: string) {
    let results;
    try {
        results = readFileSync(report_path(sha))
    } catch (e) {
        return [];
    }
    let res = []
    for (let s of results.toString().split("\n")) {
        if (!s) continue
        let entry = JSON.parse(s)
        if (entry.meta) continue;
        res.push(entry as SampleRun)
    }
    return res
}


export function get_changes(a: string, b: string) {
    let samplesA: { [key: string]: SampleRun } = {}
    let samplesB: { [key: string]: SampleRun } = {}

    for (let sA of get_samples(a))
        samplesA[sA.sample] = sA
    for (let sB of get_samples(b))
        samplesB[sB.sample] = sB


    let samples = Array.from(new Set([...Object.keys(samplesA), ...Object.keys(samplesB)]))
    samples.sort()

    let missing = 0
    let identical = 0
    let different = 0
    let identicalSuccessful = 0
    let regressions: [SampleRun, SampleRun][] = []
    let changes: [SampleRun, SampleRun][] = []
    for (let sample of samples) {
        let sA = samplesA[sample]
        let sB = samplesB[sample]

        if (!sA || !sB) {
            missing++
            continue
        }


        let objects = Array.from(new Set([...Object.keys(sA.results), ...Object.keys(sB.results)]))
        objects.sort()

        let isDifferent = false
        if (sA.statuscode !== sB.statuscode) {
            isDifferent = true
            regressions.push([sA, sB])
        }

        for (let obj of objects) {
            if (sA.results[obj] !== sB.results[obj])
                isDifferent = true
        }

        if (isDifferent) {
            different++
            changes.push([sA, sB])
        } else {
            identical++
            if (sA.statuscode === 0)
                identicalSuccessful++
        }
    }

    return {
        missing,
        identical,
        different,
        regressions,
        identicalSuccessful,
        changes
    }
}

function objects_table(sA: SampleRun, sB: SampleRun) {
    const pre = (text: string) => '`' + text + '`';
    const cmp = (a: string, b: string) => `${pre(a)} | ${a === b ? '=' : '**≠**'} | ${pre(b)}`

    let objects = Array.from(new Set([...Object.keys(sA.results), ...Object.keys(sB.results)]))
    objects.sort()
    let result = ''
    result += '| File | Base |     | PR   |\n'
    result += '| ---- | ---- | --- | ---- |\n'
    result += `| _Statuscode_ | ${cmp(sA.statuscode.toString(), sB.statuscode.toString())} |\n`
    for (let obj of objects) {
        let objA = sA.results[obj]
        let objB = sB.results[obj]
        result += `| ${pre(obj)} | ${cmp(objA, objB)} |\n`
    }
    return result
}

function make_section(data: [SampleRun, SampleRun][], kind: string) {
    if (data.length) {
        let smallest = +Infinity;
        let smallest_text = '';

        let count = 0;
        let sample_table = '';

        for (let [sA, sB] of data) {
            if (count < 50) {
                sample_table += `### ${sA.sample}\n`
                sample_table += objects_table(sA, sB) + '\n'
            }
            count += 1;

            let stat = statSync(`/root/datasets/${data}/${sA.sample}.gz`)
            if (stat && stat.size < smallest) {
                smallest = stat.size
                smallest_text = `## Smallest ${kind}: [${sA.sample}](https://arxiv.org/e-print/${sA.sample})\nSize: ${stat.size} bytes gz'd\n\n${objects_table(sA, sB)}\n`
            }
        }

        return `
${smallest_text}

## ${kind} (${data.length})

${sample_table}

${data.length >= 50 ? '' : `Too many ${kind}s for GitHub's API payload size limit. Results truncated...`}`;
    } else {
        return ''
    }
}

export function report_path(sha: string) {
    return '/root/reports/' + sha + '.jsonl'
}

export function markdown_report(dataset: string, a: string, b: string, eta?: string) {
    let {missing, identical, identicalSuccessful, different, regressions, changes} = get_changes(a, b)

    let regressionSection = make_section(regressions, "Regression");
    let changeSection = make_section(changes, "Change");

    return `
  ${eta ? `:construction: This test run is currently in progress. ${eta} :construction:` : ''}
  
  ${a} vs ${b}
  
  ## Summary
  
  | Samples | Count |
  | -- | -- |
  | Identical | ${identical} |
  | Identical & Successful | ${identicalSuccessful} |
  | Different | ${different} |
  | Regressions | ${regressions.length} |
  | Missing  | ${missing} |
  
  ${regressionSection}
  
  ${changeSection}`;
}