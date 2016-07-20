# Commands:

- ff init &lt;path-to-dir&gt;
- ff add &lt;file-path&gt; [&lt;file-path&gt;]
- ff rm &lt;file-path&gt; [&lt;file-path&gt;]
- ff apply


# Description:

`ff` is files synchronization helper.

It helps you to:

- linking files from your homedir to synchronized dir
- linking files from synchronized dir to homedir


## 1. From homedir to synchronized dir

### a. Automatic synchronization (via Dropbox/Google Drive/other-sync-service)

Let's say you have:

- file `.bashrc` which you want to sync
- directory being synchronized by Dropbox/Google Drive/other-sync-service

To add `.bashrc` to synchronization, run:

```
    # create directory where `ff` will put files
    mkdir <path-to-synchronized-dir>/by-ff
    ff init <path-to-synchronized-dir>/by-ff
    ff add ~/.bashrc
```

That's it. If you want to add another file, just run

`ff add <path-to-another-file>`


### b. Manual synchronization (via git/hg/other-VCS)

Let's say you have:

- file `.bashrc` which you want to sync
- `dot-files` directory (managed by git)

To add `.bashrc` to synchronization, run:

```
    # create directory where `ff` will put files
    mkdir <file-path-to-synchronized-dir>/by-ff
    ff init <file-path-to-synchronized-dir>/by-ff
    ff add ~/.bashrc
    git commit -am "added bashrc file"
    git push
```

That's it. If you want to add another file, just run

```
    ff add <path-to-another-file>
    git commit -am "added another file"
    git push
```

## 2. From synchronized dir to homedir

Using already synchronised files on other machine.
Let's say you have already synchronized dir and you want to reuse it

```
    ff init <path-to-dir-with-synchronized-files>
    ff apply
```
