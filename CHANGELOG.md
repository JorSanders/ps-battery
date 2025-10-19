# Changelog

## [1.16.0](https://github.com/JorSanders/ps-battery/compare/v1.15.0...v1.16.0) (2025-10-19)


### Features

* pollin is in a second thread now ([3c021dc](https://github.com/JorSanders/ps-battery/commit/3c021dcd8867c6f277dba9d29daa88e0ce97e0eb))
* using a local thread ([5e6e2ac](https://github.com/JorSanders/ps-battery/commit/5e6e2ac59e0920d6841a2c01de50b0c4520bad90))

## [1.15.0](https://github.com/JorSanders/ps-battery/compare/v1.14.0...v1.15.0) (2025-10-19)


### Features

* clean up the fully charged state ([b224a40](https://github.com/JorSanders/ps-battery/commit/b224a40041073fe0bc7065260babb9b81b05a5e2))
* detect fully charged state ([508b5ef](https://github.com/JorSanders/ps-battery/commit/508b5ef447e4640f2538d2540f187a1e4c38c005))

## [1.14.0](https://github.com/JorSanders/ps-battery/compare/v1.13.1...v1.14.0) (2025-10-18)


### Features

* only instantatie hid_api once. and way more logging ([7c9d8f3](https://github.com/JorSanders/ps-battery/commit/7c9d8f3d207b681798f94a26a7942ebed8def23a))

## [1.13.1](https://github.com/JorSanders/ps-battery/compare/v1.13.0...v1.13.1) (2025-10-18)


### Bug Fixes

* dont wrap whole show balloon in unsafe ([c937af7](https://github.com/JorSanders/ps-battery/commit/c937af70f6d0d96b96feed565c29a955626c9d6b))
* keep track of last_read_failed to remove zombie controllers ([2be457a](https://github.com/JorSanders/ps-battery/commit/2be457a888753f3e37a0ce2eb65038c3bdfdb2ee))

## [1.13.0](https://github.com/JorSanders/ps-battery/compare/v1.12.0...v1.13.0) (2025-10-18)


### Features

* add version info to tray tip text ([d0878d8](https://github.com/JorSanders/ps-battery/commit/d0878d84e399205b7c67178c30979bcbef4fa992))

## [1.12.0](https://github.com/JorSanders/ps-battery/compare/v1.11.0...v1.12.0) (2025-10-18)


### Features

* try to open device if the buffer is empty ([869a814](https://github.com/JorSanders/ps-battery/commit/869a814c88b193b1a17562ce2c0d715fdf0553c5))

## [1.11.0](https://github.com/JorSanders/ps-battery/compare/v1.10.0...v1.11.0) (2025-10-18)


### Features

* menu items for no controllers, handle ps4 feature report, safer buffer ([01635e7](https://github.com/JorSanders/ps-battery/commit/01635e7fcce4a1f885cf74ab6c8eec1a4343866a))

## [1.10.0](https://github.com/JorSanders/ps-battery/compare/v1.9.0...v1.10.0) (2025-10-17)


### Features

* log windows peek messages ([eec695e](https://github.com/JorSanders/ps-battery/commit/eec695ee9072d567872ad9068006ab60ed677f9c))
* remember controllers in case report reading fails. Prettier logging ([057d4b6](https://github.com/JorSanders/ps-battery/commit/057d4b608ba810754af3c617dfa6b039a0acad06))

## [1.9.0](https://github.com/JorSanders/ps-battery/compare/v1.8.0...v1.9.0) (2025-10-17)


### Features

* Add support for ps4(untested) ([c4f673d](https://github.com/JorSanders/ps-battery/commit/c4f673dc499a075f6294df897198d5d0ddb6b3b4))

## [1.8.0](https://github.com/JorSanders/ps-battery/compare/v1.7.3...v1.8.0) (2025-10-17)


### Features

* add logging for product_id ([3112d7d](https://github.com/JorSanders/ps-battery/commit/3112d7dc746530f21ae700bca594a612c23c1394))
* rename and format logging ([1f39c5c](https://github.com/JorSanders/ps-battery/commit/1f39c5c51af6d5e06564320e8527fb3a5f2138be))


### Bug Fixes

* dont add mock controllers to status list ([8e44aad](https://github.com/JorSanders/ps-battery/commit/8e44aada31dc6049cd62e6a20ffd3ea9cec96b4d))

## [1.7.3](https://github.com/JorSanders/ps-battery/compare/v1.7.2...v1.7.3) (2025-10-17)


### Bug Fixes

* Another round of refactors, fix clippy, and update notification msg ([f7cd237](https://github.com/JorSanders/ps-battery/commit/f7cd23753e64b591339a7c5e2cc06bfd51b5ca64))
* remove clippy warnings ([0855c22](https://github.com/JorSanders/ps-battery/commit/0855c22036ea0a7f2956a6b2e852cfd519a148dd))
* sent notification if battery is below not above 30% ([11a9fbb](https://github.com/JorSanders/ps-battery/commit/11a9fbb9d43e75648e4e48fd5d133dbca71e62cc))

## [1.7.2](https://github.com/JorSanders/ps-battery/compare/v1.7.1...v1.7.2) (2025-10-16)


### Bug Fixes

* print bluetooth without all caps ([f81a89f](https://github.com/JorSanders/ps-battery/commit/f81a89ffd28f8aaa156122ad1e624f3a2e7279dd))

## [1.7.1](https://github.com/JorSanders/ps-battery/compare/v1.7.0...v1.7.1) (2025-10-16)


### Bug Fixes

* dont print 0x before name ([f97a4ab](https://github.com/JorSanders/ps-battery/commit/f97a4abdc303306dbc1318e8737f8cdb25c9c265))

## [1.7.0](https://github.com/JorSanders/ps-battery/compare/v1.6.1...v1.7.0) (2025-10-16)


### Features

* fix and improve parsing ([80ee088](https://github.com/JorSanders/ps-battery/commit/80ee088f6b19b0f46f6f5d5c7c12611a2d20363e))
* fix parsing, add debug logging and build ([7a50245](https://github.com/JorSanders/ps-battery/commit/7a50245cf44b214ad13bd99e65e9231465cb10fa))

## [1.6.1](https://github.com/JorSanders/ps-battery/compare/v1.6.0...v1.6.1) (2025-10-13)


### Bug Fixes

* clean up logging ([4ab7a10](https://github.com/JorSanders/ps-battery/commit/4ab7a1012e73f44647d50a43d36b1b3109b573a8))
* dont panic in balloon ([cab0b7d](https://github.com/JorSanders/ps-battery/commit/cab0b7d8d1e099e77ff1aecd73b433c7e57a28c8))
* dont print (high) ([6295a1f](https://github.com/JorSanders/ps-battery/commit/6295a1fbee5cef2a9be4fc92236069257d364f87))
* dont print high ([c3a9247](https://github.com/JorSanders/ps-battery/commit/c3a9247b453e4f1d1d980af845999ccedf624532))
* i think parsing works now ([38f24ac](https://github.com/JorSanders/ps-battery/commit/38f24ac3fa2c5b37ca11c74a2ff3a61c22e6e1de))
* print extra whitelline ([c421d6c](https://github.com/JorSanders/ps-battery/commit/c421d6c5a8a20b7a9098d3e6811a1dbaa5904f3d))
* remove comment ([e6effce](https://github.com/JorSanders/ps-battery/commit/e6effce7e9d6dd1dccae4d66614a05f8c6d0acf8))
* remove last seen ([ed2c1c7](https://github.com/JorSanders/ps-battery/commit/ed2c1c7952eee3cf69c8413aa94d091a74da4fda))
* rename calibration to report ([b8aaf91](https://github.com/JorSanders/ps-battery/commit/b8aaf91dbe152cc929de07bb49579374573a8f5b))
* save bitmasks as consts ([ef94a8f](https://github.com/JorSanders/ps-battery/commit/ef94a8fbcda941bf8fff1845e3b8b5c857079f15))
* use log error with ([37e8d11](https://github.com/JorSanders/ps-battery/commit/37e8d111ed072dbe894289fd90dabc320ea43738))

## [1.6.0](https://github.com/JorSanders/ps-battery/compare/v1.5.0...v1.6.0) (2025-10-12)


### Features

* fix signing exe ([9bb6bcb](https://github.com/JorSanders/ps-battery/commit/9bb6bcba125954bca07d6736484d74833c39d01e))

## [1.5.0](https://github.com/JorSanders/ps-battery/compare/v1.4.0...v1.5.0) (2025-10-12)


### Features

* fix signing exe ([f49fe2e](https://github.com/JorSanders/ps-battery/commit/f49fe2ee2714a6787e78186adf1e706e1f9ba800))

## [1.4.0](https://github.com/JorSanders/ps-battery/compare/v1.3.2...v1.4.0) (2025-10-12)


### Features

* fix signing exe ([f373d5e](https://github.com/JorSanders/ps-battery/commit/f373d5ea02f791ce80109e9eab824940f7cc4ff1))

## [1.3.2](https://github.com/JorSanders/ps-battery/compare/v1.3.1...v1.3.2) (2025-10-12)


### Bug Fixes

* handle all results ([8d870a6](https://github.com/JorSanders/ps-battery/commit/8d870a6801de7a78b76db7bc0c2f10554824aa98))

## [1.3.1](https://github.com/JorSanders/ps-battery/compare/v1.3.0...v1.3.1) (2025-10-12)


### Bug Fixes

* remove inline if ([50f1a68](https://github.com/JorSanders/ps-battery/commit/50f1a689e9c2de256c84a9bb5da8e1d4ef72eecf))

## [1.3.0](https://github.com/JorSanders/ps-battery/compare/v1.2.0...v1.3.0) (2025-10-12)


### Features

* no manual build ([50282ec](https://github.com/JorSanders/ps-battery/commit/50282ecb30919d5fe93d1818fd343878ed595486))

## [1.2.0](https://github.com/JorSanders/ps-battery/compare/v1.1.1...v1.2.0) (2025-10-12)


### Features

* on left click open tray instead of removing ([8558fae](https://github.com/JorSanders/ps-battery/commit/8558fae0a73ea60b6623606ba65ce91015881e4b))

## [1.1.1](https://github.com/JorSanders/ps-battery/compare/v1.1.0...v1.1.1) (2025-10-12)


### Bug Fixes

* save values in controller store ([4f0d24d](https://github.com/JorSanders/ps-battery/commit/4f0d24d44dea39231e39dad2c43276451c500c6b))

## [1.1.0](https://github.com/JorSanders/ps-battery/compare/v1.0.0...v1.1.0) (2025-10-12)


### Features

* Update controller format ([b5390b4](https://github.com/JorSanders/ps-battery/commit/b5390b40c4d8b32f544a494143aabe468511a451))


### Bug Fixes

* detect charging over USB ([b9c8ad0](https://github.com/JorSanders/ps-battery/commit/b9c8ad091fcad507b068ed74a60eed535877cb8a))

## [1.0.0](https://github.com/JorSanders/ps-battery/compare/v0.1.0...v1.0.0) (2025-10-12)


### ⚠ BREAKING CHANGES

* Add LICENSE and README for 1.0 release

### Features

* Add LICENSE and README for 1.0 release ([d3ca906](https://github.com/JorSanders/ps-battery/commit/d3ca9061fe8a9463bdf5acc3a8ffc5a46734407d))

## 0.1.0 (2025-10-12)


### Features

* add failing balloon feature ([144f2e4](https://github.com/JorSanders/ps-battery/commit/144f2e423581446f17a80599919f92560f853e59))
* add toasts ([abf2e15](https://github.com/JorSanders/ps-battery/commit/abf2e15d314e9bf4eab2c7b25451d292c7027901))
* add wake up call ([a0a52ec](https://github.com/JorSanders/ps-battery/commit/a0a52ec2cdc16d386541de0230677e030051e066))
* both bluetooths ([9415fbd](https://github.com/JorSanders/ps-battery/commit/9415fbd0e555c259bd61271c93f5f344ad9dc638))
* both charging seems to work ([829a8eb](https://github.com/JorSanders/ps-battery/commit/829a8eb64c05418d1e414b1ee80703af21257af1))
* both work charging and % ([ca47776](https://github.com/JorSanders/ps-battery/commit/ca47776a7f8ba53b3e1ea6a6bd0b399b064bf7e0))
* charging works on ps5 ([b7ec178](https://github.com/JorSanders/ps-battery/commit/b7ec178a79c6ad34a11c095d9277280a391fae44))
* clean up code a lot ([77c1235](https://github.com/JorSanders/ps-battery/commit/77c1235058aca2c277e9c03a23957703746fe87d))
* clean up the program a bit ([34617a1](https://github.com/JorSanders/ps-battery/commit/34617a17e6c17e2c04f7ccfa7cd9f314b870ba89))
* clean up windows features ([658f212](https://github.com/JorSanders/ps-battery/commit/658f212ea2f4516632930aff868200bc1963df8e))
* edge works ([ffb8859](https://github.com/JorSanders/ps-battery/commit/ffb8859656d36df39463a484ac870bf41c4e5219))
* initial version using println ([63b7deb](https://github.com/JorSanders/ps-battery/commit/63b7deb59cdaf543443cb24c3d5d16d75d8bf04a))
* less consts' ([a765290](https://github.com/JorSanders/ps-battery/commit/a76529094d11dd37dc9e981578cc7af0b3845a92))
* list bluetooth devices ([32f748d](https://github.com/JorSanders/ps-battery/commit/32f748d17003979000b2e80b1aabdead6fbbe40a))
* logging ([c26da90](https://github.com/JorSanders/ps-battery/commit/c26da90f578a818b2c0d5654345715a2772427b7))
* no longer a terminal app ([cb779e8](https://github.com/JorSanders/ps-battery/commit/cb779e8a6c52e7d226c52988b3baf59b685dd10e))
* only connected ps5 ([f022785](https://github.com/JorSanders/ps-battery/commit/f022785f2891b10d06077b8f3a4dc9e1d435f9be))
* play sounds ([a535d0f](https://github.com/JorSanders/ps-battery/commit/a535d0f7e966d58dce4c7ee1d4682e20c7e0f7c4))
* run on startup ([2f2ab89](https://github.com/JorSanders/ps-battery/commit/2f2ab89727e16c925a8fafc54d47dc91e363656c))
* show statussus in tray ([36b5c58](https://github.com/JorSanders/ps-battery/commit/36b5c581e838da6c006bcbf9495237b0f08f6bdb))
* split into modules ([5f8f23b](https://github.com/JorSanders/ps-battery/commit/5f8f23b402419137b65bb3692fb72bdb229a61ab))
* split into modules ([b5f7685](https://github.com/JorSanders/ps-battery/commit/b5f7685316e1388da6eb9296d7fc527eb694e316))
* tray crate ([d750bb2](https://github.com/JorSanders/ps-battery/commit/d750bb2fa6daaf604937279d35ef5e52f9fb3c23))
* try to add tray icon ([2196ef3](https://github.com/JorSanders/ps-battery/commit/2196ef35bf1de5752c9305f5202ace56d97a87ec))
* working balloon ([a42574a](https://github.com/JorSanders/ps-battery/commit/a42574a1adb29a483af49690e512058a4949b648))


### Bug Fixes

* clean up warnings ([c9dc430](https://github.com/JorSanders/ps-battery/commit/c9dc43047e291fc44f76e7043f78574d80a26681))
* should be working now ([ac9c565](https://github.com/JorSanders/ps-battery/commit/ac9c565fe8cecd363f4143622fb18375b24df50b))
