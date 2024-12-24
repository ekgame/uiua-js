export abstract class AbstractBackend {
    printStrStdout(str: string) {
        throw new Error("Printing to stdout is not supported in this environment.");
    }

    printStrStderr(str: string) {
        throw new Error("Printing to stderr is not supported in this environment.");
    }
}