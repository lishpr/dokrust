# Dokrust

Dokrust is a container runtime written in rust. The distant goal is to run Android apps in Dokrust containers.

It is built on the foundation of [vas-quod](https://github.com/flouthoc/vas-quod). 

Now, functionalities are being incrementally added. Including:
* hostname customization
* quota assignment in different control groups for isolation
* sustained interaction within the container
* directory mounting into the container

## Usage

```bash 
Usage: ./dokrust [options] [-- <command> <argument>...]

Options:
    -r, --rootfs        Path to root file-system
                        default: --rootfs ../rootfs
    -c, --command       Command to be executed
                        eg. --command `curl http://google.com`
    -n, --hostname      Customize the name of your container
                        default: --hostname dokka
    -q, --quota         The quota of CGroup for your process
                        eg. --quota cpu:cpu.cfs_quota_us:50000
    -m, --mount         Mount directory to container
                        eg. --mount /root:/mnt
    -h, --help          Print this help menu
```

### --rootfs
Path to root filesystem. 

### --command
Container entrypoint command.

### --hostname
You may assign a distinct hostname for your container, such that distinct CGroup quotas could be set for different containers.

### --quota
The CGroup quota for your container. Should be formatted into ```{CGROUP_NAME}:{CGROUP_TARGET}:{QUOTA}```, for instance: ```cpu:cpu.cfs_quota_us:50000```. Multiple CGroup quotas should be seperated with ```::``` (two colons).

### --mount
Mount a directory outside the chosen rootfs into the container.
