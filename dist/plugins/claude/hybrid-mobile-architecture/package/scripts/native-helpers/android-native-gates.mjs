import { main, run } from './common.mjs';
await main(() => {
    run(process.env.KNOWME_PLATFORM_NATIVE ?? 'knowme-platform-native', ['verify-apk', ...process.argv.slice(2)]);
});
