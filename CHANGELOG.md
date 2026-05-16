# Changelog

## [v0.6.0] - 2026-05-16

- Add `gadd.commitFlags` config option for adding arguments to `git commit`
    - See the "Configuration" section in the README for how this works
- Use Git CLI instead of `libgit2` for `git fetch`
    - `libgit2` has some limitations when it comes to SSH authentication, which caused `git fetch`
      to fail in some configurations. See this issue for more:
      https://github.com/libgit2/libgit2/issues/4338
- Display errors from `git fetch` in the help menu
    - Previously, `gadd` just displayed "Fetch failed", with no context about what went wrong. Now,
      users can enter the help menu ('H') to see more details about the error.

## [v0.5.2] - 2026-01-27

- Update dependencies

## [v0.5.1] - 2025-09-02

- Add support for committing with `--amend` directly from `gadd` (bound to `M` key)
- Fix unstaging of all changes when there are new files in working tree

## [v0.5.0] - 2024-03-17

- Add branch status display
- Implement fetching from remote to show commits ahead/behind upstream
- Add `--status` flag to skip staging area and just print changes

## [v0.4.0] - 2024-03-09

- Add support for committing directly from gadd

## [v0.3.0] - 2023-06-10

- Add support for staging/unstaging of all changes
- Add key to copy path of selected change

## [v0.2.0] - 2023-05-21

- Add inline rendering of changes in terminal on exit
- Improve error messages
- Fix rendering of changes in merge conflict
- Fix fullscreen rendering on Windows

## [v0.1.0] - 2023-04-20

- Initial release

[Unreleased]: https://github.com/hermannm/gadd/compare/v0.6.0...HEAD

[v0.6.0]: https://github.com/hermannm/gadd/compare/v0.5.2...v0.6.0

[v0.5.2]: https://github.com/hermannm/gadd/compare/v0.5.1...v0.5.2

[v0.5.1]: https://github.com/hermannm/gadd/compare/v0.5.0...v0.5.1

[v0.5.0]: https://github.com/hermannm/gadd/compare/v0.4.0...v0.5.0

[v0.4.0]: https://github.com/hermannm/gadd/compare/v0.3.0...v0.4.0

[v0.3.0]: https://github.com/hermannm/gadd/compare/v0.2.0...v0.3.0

[v0.2.0]: https://github.com/hermannm/gadd/compare/v0.1.0...v0.2.0

[v0.1.0]: https://github.com/hermannm/gadd/compare/07ce0d6...v0.1.0
