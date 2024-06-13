import { sleep } from './testUtils';

export const pollUntilConditionMet = async <T>(
  fnToRun: () => Promise<unknown>,
  conditionChecker: (arg0: T) => boolean,
  interval = 2000,
  retries = 20
): Promise<T> => {
  if (retries <= 0) {
    console.log('WaitForStatusChange - no more retries');
    throw new Error(`pollUntilConditionMet all retries used calling ${fnToRun}`);
  }

  const result = (await fnToRun()) as T;
  if (conditionChecker(result)) {
    return result as T;
  }
  console.log(`Waiting ${interval}ms before running ${fnToRun.name} and checking condition ${conditionChecker}`);
  await sleep(interval);
  return pollUntilConditionMet(fnToRun, conditionChecker, interval, retries - 1);
};
