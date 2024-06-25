import * as R from 'ramda';

const wait = (waitMs: number) => new Promise((resolve) => setTimeout(resolve, waitMs));
export const waitFor = async (fn: () => Promise<unknown>, waitMs: number, retry: string): Promise<void> => {
  const throttledGetQueue = () => fn().catch((error: unknown) => wait(waitMs).then(() => Promise.reject(error)));

  await R.range(0, parseInt(retry, 10)).reduce(
    (lastAttempt: Promise<unknown>) => lastAttempt.catch(throttledGetQueue),
    Promise.reject()
  );
  return;
};

export const sleep = (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));
