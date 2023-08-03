# gltf-validator 

A rust library and binary wrapper around the Khronos group
[glTF-Validator](https://github.com/KhronosGroup/glTF-Validator) tool.

## Publishing a new release

We have a GitHub action that pushes our releases [here](https://github.com/KittyCAD/twenty-twenty/blob/main/.github/workflows/make-release.yml). It is triggered by
pushing a new tag. So do the following:

1. Bump the version in `Cargo.toml`. Commit it and push it up to the repo.
2. Create a tag with the new version: `git tag -sa v$(VERSION) -m "v$(VERSION)"`
3. Push the tag to the repo: `git push origin v$(VERSION)`
