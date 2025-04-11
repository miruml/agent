## Instructions
There are lots of things which need to be dynamically set in the binaries to ensure the code is running correctly and so we can easily identify the code. For instance, building the binaries requires linking the local, development, and production binaries to different server urls. And we want to maintain versions (1.3.2 for example) to collectively identify and distribute our software. The build script handles all this for you, allowing you set parameters at the top of the build script which will propagate to every part of the build. Simply navigate to the build directory
```
cd <SOME DIRECTORY PATH>/device/build
```
Then execute the script.
```
./build*
```
Three different binaries are created in the 'prod', 'dev', and 'local' folders respectively. As you can guess, one folder is for local testing, one for the development servers, and one for the production servers. The actual distribution package is only the .deb file in the directory but you can observe the contents by looking inside the miru_<VERSION>_<ARCH> folder inside the directory as well. Of course, please use and distribute the binares to the correct file.

## Debian Packages
[How to Create .deb Packages for Debian, Ubuntu and Linux Mint](https://www.youtube.com/watch?v=ep88vVfzDAo)

[Architectures](https://wiki.debian.org/SupportedArchitectures)

**Install the package**
```bash
sudo dpkg -i ./miru_<VERSION>_<ARCH>.deb
```

Another method is to use apt but this has been giving me permission errors at the very end (although the installation appears to still work correctly) 
```bash
sudo apt-get install ./miru_<VERSION>_<ARCH>.deb
```

**Uninstall the package**
```bash
sudo dpkg --remove miru
```

Another method is to use apt but this has been giving me permission errors at the very end (although the installation appears to still work correctly) 
```bash
sudo apt-get remove miru
```

