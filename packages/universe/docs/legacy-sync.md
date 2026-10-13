# Legacy synchronization ledger

## Initial shadow copy

- Legacy source: `phalcom-core/core/universe/` on branch `feat/telos-collections`.
- Baseline commit: `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131`.
- Shadow destination: `packages/universe/`.
- Initial duplication date: 2026-10-13.
- Initial working-tree status at baseline: clean (`git status --short` empty on branch `feat/telos-collections` at the baseline commit).
- Initial operation: copy of the complete authored Universe package, including `project.toml`, all authored `.ph` source, and existing ancillary assets.
- Exclusion: `src/.DS_Store` is present in the legacy checkout but ignored and untracked Finder metadata, not authored Universe source; it is intentionally absent from the shadow.
- Copied regular files: 172 of 173 regular files present on disk in the legacy directory. The 172 copied files comprise 168 `.ph` files plus `project.toml`, `src/functional/README.md`, `src/math/complex/README.md`, and `src/math/complex/CPYTHON-LICENSE.txt`; the excluded file is the ignored, untracked `src/.DS_Store`.
- Legacy files modified by this increment: None.
- Production source authority changed: No.

The manifest below records SHA-256 digests of every copied legacy file. The digests matched the shadow files at the initial S0/S1 copy; later shadow changes are recorded in the reconciliation log. This is provenance evidence for the initial copy, not proof of semantic equivalence after later edits.

## File manifest

| Legacy-relative path | SHA-256 |
|---|---|
| `project.toml` | `59743faa3342a675bbe71346014d5d5c9cf8b66f2094577c944fe503900df1c9` |
| `src/callable/closure.ph` | `fe33533acafa812156230d49243d0668a7348edd37988ebe995fc28745ecf15f` |
| `src/callable/family.ph` | `255bbffa63aa9d19e0523c181b7d7b9387007952ff80cd1c0f1331bd48b9d6e6` |
| `src/callable/function.ph` | `684af51cfe58a0125f637866e0088277d590b845c6542b4e072e960ee68da663` |
| `src/callable/method.ph` | `0a67851c89a26ac15b1df79ed8271de4e5e865bb335fe99bfdab61f8a4fbd6ba` |
| `src/callable/package.ph` | `9540a2d0551023c4642dbe9e75efe122f7bd1c75bfa7e981ad686c002579b6f2` |
| `src/collections/array.ph` | `188f4f1cc970d3ae28de32fec61a7e48d5fe3bd754167e6378b9bf4cfa48d032` |
| `src/collections/bag.ph` | `2be25fc3492c1958f0bdc4a2ce6dd937204ac081e78df16d598562e24d84672f` |
| `src/collections/bijection.ph` | `9156121d27e45d7d66968a7de42b97cd0155d83b151b296082a08dbcd326481f` |
| `src/collections/buffer.ph` | `a36064e989623132fc2eae37fe2d6fcb3b82304cfd05e2fecc249bd81e5af4af` |
| `src/collections/bytes.ph` | `612e60bfa6311515a17696df6d587ac47c1eccf68691ed4130ce5ae001090aac` |
| `src/collections/collection.ph` | `d6140b8b96d6c92194e6501b8606e2acac2b6d89a1d3fab7ad15ee9ed3238b10` |
| `src/collections/indexed_dict.ph` | `9e33aaf5a8de800ceebb7a22d142345c8dfa9cdc88dea7e0c465049b963152f4` |
| `src/collections/interval.ph` | `486da25b7e403483949a79606eba079fbe400287bbd02d46b86c94ae31126aa3` |
| `src/collections/iterator_laws.ph` | `579442979fd801ae1c6f012532dfa5fb7cc794657f0b833bc382c5345df2d11e` |
| `src/collections/list.ph` | `63f67d48aa660a40fbb3eda431d517e36b52b2247fb0cc16f0d288495154f8b9` |
| `src/collections/map.ph` | `737f060470d077e1ff94cdf540d2764fdb48ee44abda5103ced18469b2125dbf` |
| `src/collections/mutable_bag.ph` | `2c2e8cab5c1d730f5c4e16e3cf4fcddbdef168ef83831b15e005e146c95c8fd2` |
| `src/collections/ordered_set.ph` | `7266161e71851cc5f45c34b7af001e81adda55ee007971b0c852263ab3090c9b` |
| `src/collections/pack.ph` | `8689e803e863c63dc84537b9c44d78d69e8e44103bc69a183724c74c2a0d4667` |
| `src/collections/package.ph` | `b17ed0904150b1a3491fed5d6639eb9b874b3827ca589fe4aa1acaac993d43f9` |
| `src/collections/range.ph` | `a87b3125c402d384fc5764c037e5cfbc502bbe203e5410cae8bbf386cea85e29` |
| `src/collections/range_map.ph` | `befd891772b166d262277f787dd9ed10ca3955ab2a6e07dc0c29821e94d08e3f` |
| `src/collections/record.ph` | `716c478308afc0544d0861fd8eb59c1d95fe3df1bd7c9358526be95ef09c3832` |
| `src/collections/set.ph` | `a1934cadd1a6a9814109764f2800b617f4269bf3ee463eb3ff9d8c449a39d644` |
| `src/collections/slice.ph` | `a40b04660f4f0efd1314fd15bbdb341fe1123ed6f74e34cdfab16a2286a06916` |
| `src/collections/tuple.ph` | `161113b0f1a673b68fdf6d420480084d5796a369a4f2cfeb1308bd53cdce8379` |
| `src/collections/vector.ph` | `7d3584e99e232e07df91fa6ed1cd7617cfd9eeaf73e518d8660393d01d2b9ae8` |
| `src/concurrency/fiber.ph` | `7706aab05835b593532b3b277e6fc8a4a0d0398316fa6dacb3c924143d9b0fcf` |
| `src/concurrency/package.ph` | `69377f37157b96f083b93479371a707d0c692972e612fea630c7c55c7146e246` |
| `src/concurrent/package.ph` | `da6ddc756c51c10c6a3f2ad79e0c1c6a5a3ffcc6601247ea1fff3b7ee7dbec7a` |
| `src/errors/argument.ph` | `67dec3d01f3d5edcba324e50141ab416b4ff27751664b5f95cf8d0bcf24a6069` |
| `src/errors/contracts.ph` | `ea696c708fda3e9626e199bd26ae42b1ec329412bc5bcc38fa4be68d08a1c62b` |
| `src/errors/error.ph` | `51fa003c92b1f8ee5125f8c80cbffb6f002eb9d0501fb7daf65cb70f5e8d8d87` |
| `src/errors/indexing.ph` | `c8f042c0c49b5d708cfc7101087d574487cb1b31b253b01a0e9f851e3dc8ad63` |
| `src/errors/iteration.ph` | `8b2f90bee9d62a833e3c0040c3a5d4cb1c64ba43a340b30aa938b5857bc2e8dd` |
| `src/errors/package.ph` | `1b7f47e8b24a1751642fcf9ed709b9121817f315b37665698433e7f175bb83a4` |
| `src/errors/result.ph` | `9b525f264282b77e81112fed2eaa2ecfbb69c4815269061bc12e02e432aa417b` |
| `src/errors/unimplemented.ph` | `3679a766f86fb573219c0f1968c50fc172476532ac76a28b54c9338032f0586e` |
| `src/errors/unsupported.ph` | `3b0faa7013fd9f764b93e5cda7b81d0a7325ef166af69011baeed3cd384d7a93` |
| `src/fs/package.ph` | `d4126bdc54ad5ef8a3f2504ec902fb93f3c487c853df66d45072d8cbedf0d65e` |
| `src/functional/README.md` | `a341af8edfe2589b1f44fff70c085623b3550d8fbfb7eb49a02f58869988e951` |
| `src/functional/algebra.ph` | `6eeb62dc11c7025fe00965f509fea04c7ed499d5fa08c8237da35c0bf703fd9d` |
| `src/functional/arrow.ph` | `48bfe2b7a807658d86d88de3dac7fb2007f447c627287493da2261d3dfa8b427` |
| `src/functional/chain.ph` | `db838b5cf10a91e99a8ea25edcd2150c8813d05eaf66fb98cf508fb34dbdc325` |
| `src/functional/effects.ph` | `cd5ef852df3c827b89442925e73e261fa3a7b5cbc0aa3e93f55fcaab4922f25e` |
| `src/functional/either.ph` | `73d293826288b6faa1c636a25b6fe128788c8772336cbc44298d4af81ef75111` |
| `src/functional/fold.ph` | `c4418107295deb6df786afda4a072a0330cc2a2b97022c8bc991481044b60d6e` |
| `src/functional/folds.ph` | `99704538ddb13e7735e0b721d9dd09700dde684f78cb954c2c50589ed8d1b059` |
| `src/functional/higher_kinded.ph` | `61dc286d5576a4d3adcb4c2190cd3ed5055b0238ee54626472291a2f32e592e2` |
| `src/functional/journal.ph` | `021a3027f1fbc2230a9c6ba8d7073101f95465540f0ea6ea2da289e05930113f` |
| `src/functional/lens.ph` | `4078774aab7bb40139bc9a04986a0426fa7d53e3108a20251048ad78d9e8130e` |
| `src/functional/optics.ph` | `67aed3cff74133bd826bbcebe7e1a2711f8e62963e9ab7f5f03a7a0ddcba70bc` |
| `src/functional/package.ph` | `27b79857a3aba70c342137d9261851343666b62e72a27f5a7e3156bab6514834` |
| `src/functional/reader.ph` | `97935636589462d000a515149a069c6213708d4c90acc952d3bbcc71c6d2cd10` |
| `src/functional/reader_result.ph` | `cd7e22ae3f6d627df09b4223f08fdc7b1a50f72c01a1a5e174daad3cb2937687` |
| `src/functional/state.ph` | `c457651697ccd71ebe2e4a5e7bf8541839ed2464fcae2b450e6ae20df4be663d` |
| `src/functional/state_result.ph` | `c35e1afdc494f6b6634d7ecfe2b9cafaed757a3a6a4a6b31fcbc356c7df5f695` |
| `src/functional/validation.ph` | `65cdb81006fa8423204e9b830f493216f7197618b3eef0c0bf0bec5122c37e7f` |
| `src/functional/writer.ph` | `3893e7fd29cddc5a3fd8db27574db2271dc1625da9933c3bafec56f4dcfa4a41` |
| `src/io/package.ph` | `4e510d1fa6554fa658900b88630e851462b66f7f307f7ae717a7fe6ceb1cdcfe` |
| `src/json/package.ph` | `e8a9cc474678f92c2fda1814aad57b65f1f5ecf08b690655227d7fb50b4f03af` |
| `src/math/complex/CPYTHON-LICENSE.txt` | `d57bc159fb2d10ea9aac494b8af822a496282fc1fe5e370ed1ce3e813635e0b4` |
| `src/math/complex/README.md` | `00c978bf4c94023d10d959f8695d221bb761f1e34078df01c49fd91eb6f1dc46` |
| `src/math/complex/kernel.ph` | `bcb3c4bc715dad7de4958a846aea14956b22472b03a1c818abfc459935c37a77` |
| `src/math/complex/package.ph` | `c8d88c8738f072c09c7fbb51ac86d5d859f6634a9abecd1f13c1098659d2fd0a` |
| `src/math/float_kernel.ph` | `80b2f309bbbb2619c8401a3ee017d5ccc1fb32a7c08c4f3ec9c2bdd49b57fce0` |
| `src/math/integer.ph` | `6571fb9a16b20a35a30a4c9d0af1ab427d7d1b1dfdbd2f751f3b3ba519b9ab90` |
| `src/math/numerical_errors.ph` | `92b55865130ba39b707967b49918d085bd7340aa5f3ffeae90604cb838670eb0` |
| `src/math/package.ph` | `79bfd186deb5130e3e0e797da5a6e6ad498dba98a4ab45f4b1c8c7767cc9aa52` |
| `src/math/reduction.ph` | `12e958f6af046fbe61ba4418d6da59a681bc0012b544b2660aeeca6e69e5239c` |
| `src/net/package.ph` | `fc5589d20d863474cec2439eaa3709704cca2083e2a8628054ccb843e8298cb8` |
| `src/numbers/algebra.ph` | `1a447c2ce7c036982cfcad72514e56e9268a7b786179cb08274305aee01c9bc1` |
| `src/numbers/capabilities.ph` | `6d6b8341e5280c8ae3a61a87736bd4aeaf647dfc4e1292bb92138aaff68bd558` |
| `src/numbers/complex.ph` | `eddcdcd350627ed6aec7e6c242a9de6d8c692ee9a6c5528785fbaaa316916d57` |
| `src/numbers/float_complex.ph` | `040c285d7551762eff0c9403387df3ccd18d1d7b349a5214d8da7fc332fdf6e7` |
| `src/numbers/package.ph` | `0775d91a2cbd77de865dd72cf29b1199c3374de09b9a32fd18982d7800c8018b` |
| `src/numbers/quaternion.ph` | `4dfe7f603282b1beefbddeb5ed4c5368933fab5f68112777028a506331cdedd8` |
| `src/numbers/rational.ph` | `391760bda00e74da70967056f3d165ef558df68155b8c19feedf95c14998f5cb` |
| `src/object/behavior.ph` | `9e98bddace1bac446706c27c8359d2bc0137e802e321850464ecee25774a8058` |
| `src/object/class.ph` | `90e5968ac01554f5b4fb26c74342191fa7c0204b1e0641962cebf083593267f2` |
| `src/object/ellipsis.ph` | `d9b33c9393f1eb2d98c340ec0a98a6911d735255329996a83e7ee4ddcfe1afe5` |
| `src/object/metaclass.ph` | `cd736edbfd549105f56f501602adc94e920ea95c5958be6c6f2a91f7ca8b4280` |
| `src/object/object.ph` | `1483bc35c89a1fe0e3c70850c679255d170335ed605e0bd9db4c6d66019659e7` |
| `src/object/ordering.ph` | `3a1529749f16df6d16d74394b79f7644b7706481ef75a145965f7e9b4e406cbc` |
| `src/object/package.ph` | `c5c4ff35cc443b403a6bf83ab40ca6c68734b1d1eea41024567ae59c8762cc9c` |
| `src/option/option.ph` | `2de162a223613c83992281d3febfdd2e8924667c1f337675fbe0f4edf12eaece` |
| `src/option/package.ph` | `bf3f11dcd162fd0274ff9f39fe91b35fdbcdfe1fb760733207e4e94ee24eae45` |
| `src/option/unit.ph` | `9b1da1fbebc4cf5f087273831bb100deaa3d44717e59f9d7b9c38e1d5b106a83` |
| `src/package.ph` | `5a2c6594e3cd9c95bdece6bf409f43798ba68f0f68701382e01633baa8a529e3` |
| `src/path/package.ph` | `4661f20ce6fc4f361c22b8cbb5661277a95343f69ab5ffae68d25e6de1160ff8` |
| `src/process/package.ph` | `34d1a38c15a862fc6f83101de8b651b2b46bbafe5d2d3e2c55ab086100ad832f` |
| `src/random/bernoulli.ph` | `718e90e1c5627504aa7efe50b55414204f9ca543ae46e31a9922d811ed6d5633` |
| `src/random/distribution.ph` | `4ff568d03e8e9f10610fec65d4ab2418a07b13d62989940f395ccb216fbf3b25` |
| `src/random/generator.ph` | `14f137a809509ef478907b8f6d5aeee7dc49df82c99a67cba93b47ebb23af55d` |
| `src/random/package.ph` | `172198e5ee4e6ac1a3f577566e1c31dba5d4cb0dc713c4b691853d851a895c1a` |
| `src/random/secure.ph` | `3403ebaed3a4b9e1c2be0bd6299f5296656d711b559c90da7910acd3cc8e0d18` |
| `src/random/uniform_int.ph` | `5b92a85427cdaf8347c9e7e15b09e42addebcbe85bda9510344830d4a0374c2b` |
| `src/random/xoshiro256pp.ph` | `bfdbd0a426d2724e0249a4a4114fc19a0cf036ed1347ea2dbc067838175fcd96` |
| `src/reflection/attribute.ph` | `415cf3ab9c3dcd9f2c3ae523b4b13c66e6760cc51555452cfe17db0ceee7502d` |
| `src/reflection/child-module-table.ph` | `4346f1f58e3c1f91a2492c2ae6d6fc6c14e26f2ea6ed3fa3b4f8a9dd21d2b66f` |
| `src/reflection/export-kind.ph` | `d655b54a828f38ef0d51dc7c9a96e922fac16df81773fc06714e06f810f62bb7` |
| `src/reflection/export-table.ph` | `87d34bc92e20f1d05e46318d6e6f9ef1a7c58b09487b16990113dfe187c5de19` |
| `src/reflection/export.ph` | `ff64f810d8ba4d4634d18f371ef48192a5ac257bc455ec8346909374d8ec56f4` |
| `src/reflection/implementation.ph` | `55242404315183103437bef4ede06ea913bb24a6ac09852a989ffffa0d00b1f6` |
| `src/reflection/message.ph` | `b3b71f8e2ae227ef95e6d0c7d609b7f2a99df6f89817523483fd197d5752f19d` |
| `src/reflection/module-dependency.ph` | `f1318f4163dba1231da2f7649869b428e598e2e6a3b7b87c87e8e18e2e24a29f` |
| `src/reflection/module-identity.ph` | `4f9c49a4aa4daea0e005b562ebb7dffddabdfe45e02295701e9c0eb5138b6955` |
| `src/reflection/module.ph` | `8868bd589d66a8dab9a6f12e7130babbe0728c2e70ade3c17fe82513e751a642` |
| `src/reflection/package-author.ph` | `24480998ce7be836449dd7ea6961adec025ad2556848dc1f883b992b617a30d3` |
| `src/reflection/package-identity.ph` | `250d3cd7e4666a6252c3ebdf6229db4d86eb848c5031be225186f380369157cc` |
| `src/reflection/package-info.ph` | `0f02db43689b174f4a54160ebbf7c19c44af16c207328c8eb30b5e48ce493e8a` |
| `src/reflection/package-object.ph` | `c2f3631509bfac46bc4bf571a386845910748ca8f9fc1ac296a53b0f600b0b3d` |
| `src/reflection/package-requirement.ph` | `21eb8aa950716f969001ac0c72b340e60926a17b9b443edc55b5ba1043397d66` |
| `src/reflection/package.ph` | `6667f725e5b436b190414df822046dfdcb9b0e887300cf58e5e483c57d525a4c` |
| `src/reflection/project-identity.ph` | `41ba2da010da2e97b2faf359b050f8a81df8ef0372eb222bfb65ad2310ce2a6c` |
| `src/reflection/project-manifest.ph` | `aaee5ad19caf0fd8f0ee16da3b1320df377cd62579bff74261b255d1f88938c3` |
| `src/reflection/project.ph` | `940f4638df825c29217e257aa21c012d9c9e4585552845ec3ecb4f00b9cf6985` |
| `src/reflection/resolved-project-dependency.ph` | `4749f84fc0ea1647c7519ee0c36338b2cdb8ba97cf24f007026308b9766fb63b` |
| `src/reflection/selector.ph` | `a043425ab3eb37a42a93778e6d45b12f2f6a950254fe87b4ab839224c22294be` |
| `src/reflection/structural.ph` | `1523544aabbb43218ca0f0da233aa72d3176589abeaa39ba774cf3cf0e1c2f31` |
| `src/reflection/typing/context.ph` | `468f3f9aea5f4ced994576b027d19e53c3cf7431639e87460fda9ec38699bfdc` |
| `src/reflection/typing/evidence.ph` | `74ac0e2ce7d26fa4714bede9695885ea81ae35ea311cec631e2dd18843dcb3f8` |
| `src/reflection/typing/generic-signature.ph` | `41bc1a4707bc9becd8e0036b5c9cd21d5c1123150461a29e3c07f44be1afc9e8` |
| `src/reflection/typing/kind.ph` | `6dd709deb381392bcbd3237a7aa29a5dc34a17f45341b9b7576b19740d669a96` |
| `src/reflection/typing/package.ph` | `9a71235c376372fafdcd9fbce5171eb49ff483c765c63cfff6d0a8dc5400d81d` |
| `src/reflection/typing/result.ph` | `8d88bbbbdfba1213d11507c4c75a7ea260be2c6fe5f636d1409ecc521ceae9d4` |
| `src/reflection/typing/signature.ph` | `a3ae35613d654277f5922f15941b53061b8b324bd0a75952f1b7e461ce8a9d8e` |
| `src/reflection/typing/type-descriptor.ph` | `4a5c52ff4eeaae2c14a05555d7e6a909088022dcf096855a7e3391bf6ee8f99f` |
| `src/reflection/typing/type-parameter.ph` | `695902c2ac37a166fbbc85e5f2bb8a9e397616aa93228ca4e6ff0a252f9f3076` |
| `src/reflection/typing/type-use.ph` | `71f0e6b9f267f5f347d3f99ab1c9ff05f767a37cf564a59fc34ca7774c5ca706` |
| `src/reflection/uri.ph` | `dbd55a2703d2312b958093ecda2e42c28cb6d900e97077cc71c2c7d49afef3d5` |
| `src/regex/package.ph` | `ca76966002711d9512a26717985eeb17455e8344c1250c7105595e42bb59c396` |
| `src/scalar/bool.ph` | `ec52e3e497f17e47e7a726efe65e307382b005301f770c8cea1e60e04171f8c8` |
| `src/scalar/char.ph` | `c8db0afad5832a709d2bd3b97cf52df4b9db6a9fd294cf2056cf1c3b4d9316ec` |
| `src/scalar/nil.ph` | `8f27df9e5e5560aa80e7c5a4df007c543abd6b32d196259be1412c30ecf57c9e` |
| `src/scalar/number.ph` | `cb4d4b479fdb0884bb53985ae9ef38dc54080b5261ec0781f0cc755ff2e96586` |
| `src/scalar/package.ph` | `d757afdc7a91f063b097a9d5888ef8405fec48366e3503f13baacc8b26c6bbfb` |
| `src/scalar/string.ph` | `78c67bab812a7faf411118fb367af25ec8c585d59802c3ae5fbf19c259b96515` |
| `src/scalar/string_index.ph` | `1419ef88de5218aaf775ae789e5bcb82acf7089c15458e2db1d6cc967ed6cda8` |
| `src/scalar/symbol.ph` | `8d304398c1627de192b3fca6e6f51702ef8a2deb2459917404f26cd45c0211d4` |
| `src/testing/package.ph` | `f4ca8c3d46f4242554c46bc253b966b18c03927562ecc23e95a724389424120f` |
| `src/text/builder.ph` | `9e71eb6aee4e1dada09c71dfa2d1e57783df17e97ff5e47febb8ba123948e022` |
| `src/text/cursor.ph` | `965a7edc2ef90a2c4faebb905480b8ec7377fce374b5679a66bbbecb9978b1c7` |
| `src/text/errors.ph` | `20d29b9754bc5b1c29149dbf62a3c3027da89b68051fbd2fd027e1ab53c5edaf` |
| `src/text/line_map.ph` | `dbf0eea00127a19938f41a5b04b0a872b14c35f9d76e7f034ab3d435b9d9aa09` |
| `src/text/lines.ph` | `ce2a993f91ec84c23bb5f034301923667314e4e97d3a71364400941555dd72ec` |
| `src/text/package.ph` | `82814ea4c0bf92439b20fae7d34b8edeaa068a670fc96783f4cd2d4c930cb128` |
| `src/text/patterns.ph` | `c4292caff6dbe9b03d7d55f3a1c84475161e35469c2d50a6bbbba2cd1d268925` |
| `src/text/split.ph` | `28500c57af48eacb372a50c4fded23b669f41477307aebcfd65087843aac72c7` |
| `src/text/substring.ph` | `2ab88b6dbc602f01359fe0daafa5adb4451ba9da70a40f7d7928a31621589c44` |
| `src/text/traits.ph` | `55439f7892f68bd688696cb6be7f909b3f6c7714ac765e9cbd5e30f886832f45` |
| `src/text/writer.ph` | `7e4082e0e1b9bfb0cedf57a29f9ff6ed76579df1b5f6aa478cd43d5cc719a04e` |
| `src/time/calendar/date.ph` | `5970fb01a92bf9cd49d51511fb660cc7874f9cb5f455b32f6a4b7be8890d5d96` |
| `src/time/calendar/datetime.ph` | `2977d6e08dcef62bbd53b4938c327569cc0dca24480c98e87907c41376ccf547` |
| `src/time/calendar/month.ph` | `63df97b217fb4ae9258d9513d3840122bfc694c276b4202fd342fbae8856456d` |
| `src/time/calendar/package.ph` | `4710f97c7a125ca5e840fd6194b41da473b682decda36bfe6c861f2558311909` |
| `src/time/calendar/partial.ph` | `ce89ef707acc5f73c6bcf6e6a6ae885ca4d72a4e5ce9ee97b03540f4ca9250ce` |
| `src/time/calendar/period.ph` | `e19be323350ba5c96e9771a707a34ccdb52e73b3f1a50c80afa35b6bd8143dc3` |
| `src/time/calendar/time.ph` | `3a5d7cc941d8bd046155ca9a676d11ce07dc4f7590139c446df6f6a595dcc00b` |
| `src/time/calendar/weekday.ph` | `5353cc2de6382e9091c67b9b0f2312a937c59d66cf704df9c4dc4753e9d5a0d4` |
| `src/time/clock.ph` | `ddbe96c492d802cd25dd42f4458cadd2f3b2fe0dd831ac8b220001bf2d5c8c66` |
| `src/time/duration.ph` | `c9b6b5df85405d13e897ba2bb7a5559a6e44f3bce3a140208f465a08b1c99d4a` |
| `src/time/offset.ph` | `2e80475f239d93013adbb8f757e787dc88457ae309923ca62508bdc1eec6a715` |
| `src/time/package.ph` | `0e1ec50c32a43fa3bbe10fec30ce9153ad587fed031e43d1ac12b5c13968428b` |
| `src/time/resolution.ph` | `57662e6de281de971cb4769bc7bb950f3215330c5f35b12f529ac34e84f4ac10` |
| `src/time/testing/clocks.ph` | `f46293c04f375d81e03f3122c8d85814b44f01f2dff7d7b338247b83d17438f7` |
| `src/time/testing/package.ph` | `389f8c58d701e7065549133d15a348e313f7a55b08b7fe3338a105b2ff8b6a1b` |
| `src/time/timestamp.ph` | `75b6dd951f7aa13377130c0abdb9eef76fdf11453f1c262d5edf90149c431a0f` |
| `src/time/timezone.ph` | `beff916971d6a506ae2f3bd7502b6e7a2bc79a29ab39e950ef7c8fb4520c339a` |
| `src/time/zone_transition.ph` | `a931b46791942c13cadef8a6017c87407a9407f28ba65d7c3e32e432a8489b5e` |
| `src/time/zoned.ph` | `3e4b1d4dec70ddcd2dc39e492dce6649dd391398808bf554d6026970181017b1` |

## Reconciliation log

| Baseline/revision examined | Shadow changes | Legacy changes not ported | Conflicts / notes |
|---|---|---|---|
| `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131` (initial copy) | Complete authored directory copied unchanged | None known at initial duplication | No semantic reconciliation performed; source layout remains the legacy layout. |
| `4d98eb53` (S3 scalar split start) | Moved `Int` and `Float` declaration bodies into shadow `scalar/int.ph` and `scalar/float.ph`; retained `Number` in `scalar/number.ph`; added explicit declaration exports and scalar-facade re-exports | None; the legacy tree remains unchanged | Production native keys, prelude ownership, runtime attachments, and the normative Universe spec still point `Int` and `Float` to legacy `scalar.number`. The shadow relocation has source-level evidence only; compiler/runtime identity was not validated. |

For future synchronization, reconcile declaration or method changes into reorganized shadow modules individually. Do not recopy complete legacy files over reorganized modules.
