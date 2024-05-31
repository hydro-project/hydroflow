export function getLines(str: string, lineStart: number, lineEnd?: number): string {
    let lines = str.split('\n').slice(lineStart - 1, lineEnd || lineStart);
    const leadingWhitespace = Math.min(...lines.map(line => line.search(/\S/)).map(Number).filter(n => 0 !== n));
    if (0 < leadingWhitespace) {
        lines = lines.map(line => line.slice(leadingWhitespace));
    }
    return lines.join('\n');
}

export function extractOutput(output: string, short = false): string {
    const outputLines = output.replace(/\n$/, '').split('\n');
    outputLines.splice(0, 4);
    if (outputLines[0].startsWith('%%')) {
        const count = outputLines.findIndex(line => 0 === line.length);
        outputLines.splice(0, count + 1, '<graph output>');
    }
    const stdOut = outputLines.join('\n');
    if (short) {
        return stdOut;
    }
    return `#shell-command-next-line
cargo run
<build output>
${stdOut}`;
}

export function extractMermaid(output: string): string {
    const outputLines = output.split('\n');
    outputLines.splice(0, 4);
    const count = outputLines.findIndex(line => 0 === line.length);
    outputLines.length = count;
    return outputLines.join('\n');
}