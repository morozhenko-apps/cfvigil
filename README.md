# CFVigil

CFVigil is a free, open-source desktop companion for watching Cloudflare deployments. It stays in the system tray and makes it easy to see what's happening across your Workers and Pages projects without keeping a browser dashboard open.

## Why I built it

I maintain several apps and games, and a number of them rely on Cloudflare. When I'm working on a release, I don't want to open each project individually to find out whether the latest deployment succeeded, is still running or needs attention. I just want a small, clear view of everything, and the option to open the details when something looks wrong.

That's what CFVigil is for. It keeps routine checks out of the way and makes problems easier to notice, without turning deployment monitoring into another full-time task.

## What it does

The tray card gives you a quick count of successful deployments, deployments in progress and issues. Expand it to search your resources, see their latest reported status and open the relevant project in Cloudflare.

CFVigil follows the **latest deployment attempt**, rather than assuming that the last successful version tells the whole story. A new deployment in progress is shown as in progress, even if the previous attempt failed. Missing or unavailable information isn't mistaken for success.

It can also notify you about changes that need attention, keep a searchable local event history and highlight unusual Worker error rates or WAF blocking activity when Cloudflare makes that data available. You can set a refresh interval, choose an interface language and optionally start the app when you sign in.

CFVigil works with Cloudflare using a **read-only API token**. It doesn't publish deployments, change Cloudflare settings or manage your infrastructure. The token is stored using your operating system's credential service, and application preferences and event history stay on your device.

## Platforms

CFVigil is a desktop app for **Linux and Windows**.

## Feedback

Found a problem? Send a [bug report](https://github.com/morozhenko-apps/cfvigil/issues/new?template=bug_report.yml), ideally with the steps needed to reproduce it. Have an idea that could make the app more useful? Share an [improvement suggestion](https://github.com/morozhenko-apps/cfvigil/issues/new?template=feature_request.yml). You don't need to know how to code the solution.

Please remove private project information and API credentials from screenshots and logs. For security issues, use [private vulnerability reporting](https://github.com/morozhenko-apps/cfvigil/security/advisories).

CFVigil is maintained independently. Reports and ideas are welcome, but I don't accept outside pull requests. You can fork the project under its open-source license; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for attribution.
