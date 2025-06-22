# AFPack

consolidate artifact directories like `node_modules` and `target` into sparse disk images + mounting them as volume in the same path so its totaly transparent

introduced in macOS 26 Tahoe

> Apple Sparse Image Format (ASIF) files transfer more efficiently between hosts or disks because their intrinsic structure doesn’t depend on the host file system’s capabilities. The size the ASIF file takes on the file system is proportional to the actual data stored in the disk image.
> 
https://developer.apple.com/documentation/virtualization/vzdiskimagestoragedeviceattachment/

