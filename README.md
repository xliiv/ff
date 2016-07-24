# Commands:

- ff init &lt;path-to-dir&gt;
- ff add &lt;file-path&gt; [&lt;file-path&gt;]
- ff rm &lt;file-path&gt; [&lt;file-path&gt;]
- ff apply


# Description:

`ff` helps you manage dot files by:

- linking files from your homedir to synchronized dir
- linking files from synchronized dir to homedir


## 1. From homedir to synchronized dir

```
ubuntu@ff-test:~$ ll
total 20
drwxr-xr-x 2 ubuntu ubuntu 4096 Jul 24 11:01 ./
drwxr-xr-x 4 root   root   4096 Jul 24 11:01 ../
-rw-r--r-- 1 ubuntu ubuntu  220 Jul 24 11:01 .bash_logout
-rw-r--r-- 1 ubuntu ubuntu 3637 Jul 24 11:01 .bashrc
-rw-r--r-- 1 ubuntu ubuntu  675 Jul 24 11:01 .profile

ubuntu@ff-test:~$ git clone https://github.com/xliiv/dot-files
Cloning into 'dot-files'...
remote: Counting objects: 3, done.
remote: Total 3 (delta 0), reused 0 (delta 0), pack-reused 3
Unpacking objects: 100% (3/3), done.
Checking connectivity... done.

ubuntu@ff-test:~$ cd dot-files/
ubuntu@ff-test:~/dot-files$ wget https://github.com/xliiv/ff/releases/download/0.1.0/ff
ubuntu@ff-test:~/dot-files$ chmod +x ff 
ubuntu@ff-test:~/dot-files$ ll
total 1320
drwxrwxr-x 3 ubuntu ubuntu    4096 Jul 24 11:03 ./
drwxr-xr-x 3 ubuntu ubuntu    4096 Jul 24 11:02 ../
-rwxrwxr-x 1 ubuntu ubuntu 1333184 Jul 21 19:33 ff*
drwxrwxr-x 8 ubuntu ubuntu    4096 Jul 24 11:02 .git/
-rw-rw-r-- 1 ubuntu ubuntu      12 Jul 24 11:02 README.md

ubuntu@ff-test:~/dot-files$ ./ff init .
Set tracking-dir to: /home/ubuntu/dot-files/.

ubuntu@ff-test:~/dot-files$ ./ff add ~/.bashrc 
added: /home/ubuntu/.bashrc (to: /home/ubuntu/dot-files/./homedir/.bashrc)

ubuntu@ff-test:~/dot-files$ ll ~
total 28
drwxr-xr-x 4 ubuntu ubuntu 4096 Jul 24 11:06 ./
drwxr-xr-x 4 root   root   4096 Jul 24 11:01 ../
-rw-r--r-- 1 ubuntu ubuntu  220 Jul 24 11:01 .bash_logout
lrwxrwxrwx 1 ubuntu ubuntu   40 Jul 24 11:04 .bashrc -> /home/ubuntu/dot-files/./homedir/.bashrc
drwxrwxr-x 4 ubuntu ubuntu 4096 Jul 24 11:04 dot-files/
drwxrwxr-x 2 ubuntu ubuntu 4096 Jul 24 11:03 .ff/
-rw-rw-r-- 1 ubuntu ubuntu   61 Jul 24 11:06 .gitconfig
-rw-r--r-- 1 ubuntu ubuntu  675 Jul 24 11:01 .profile

ubuntu@ff-test:~/dot-files$ tree . -a
.
├── ff
├── .git
    # .. cut usless output
├── homedir
│   └── .bashrc
└── README.md

21 directories, 26 files

ubuntu@ff-test:~/dot-files$ git status
On branch master
Your branch is up-to-date with 'origin/master'.

Untracked files:
  (use "git add <file>..." to include in what will be committed)

        ff
        homedir/

nothing added to commit but untracked files present (use "git add" to track)

ubuntu@ff-test:~/dot-files$ git ci -am "added bashrc and ff"
ubuntu@ff-test:git push

# .. cut usless output

```


## 2. From synchronized dir to homedir

Assuming you have already dot-files repository with `ff` included

```
ubuntu@ff-test:~$ ll ~
total 20
drwxr-xr-x 2 ubuntu ubuntu 4096 Jul 24 11:31 ./
drwxr-xr-x 4 root   root   4096 Jul 24 11:31 ../
-rw-r--r-- 1 ubuntu ubuntu  220 Jul 24 11:31 .bash_logout
-rw-r--r-- 1 ubuntu ubuntu 3637 Jul 24 11:31 .bashrc
-rw-r--r-- 1 ubuntu ubuntu  675 Jul 24 11:31 .profile

ubuntu@ff-test:~$ git clone https://github.com/xliiv/dot-files
Cloning into 'dot-files'...
remote: Counting objects: 8, done.
remote: Total 8 (delta 0), reused 0 (delta 0), pack-reused 8
Unpacking objects: 100% (8/8), done.
Checking connectivity... done.

ubuntu@ff-test:~$ cd dot-files/
ubuntu@ff-test:~/dot-files$ ll
total 1324
drwxrwxr-x 4 ubuntu ubuntu    4096 Jul 24 11:32 ./
drwxr-xr-x 3 ubuntu ubuntu    4096 Jul 24 11:32 ../
-rwxrwxr-x 1 ubuntu ubuntu 1333184 Jul 24 11:32 ff*
drwxrwxr-x 8 ubuntu ubuntu    4096 Jul 24 11:32 .git/
drwxrwxr-x 2 ubuntu ubuntu    4096 Jul 24 11:32 homedir/
-rw-rw-r-- 1 ubuntu ubuntu      12 Jul 24 11:32 README.md

ubuntu@ff-test:~/dot-files$ ./ff init .
Set tracking-dir to: /home/ubuntu/dot-files/.

ubuntu@ff-test:~/dot-files$ ./ff apply
symlinked: /home/ubuntu/.bashrc -> /home/ubuntu/dot-files/./homedir/.bashrc

ubuntu@ff-test:~/dot-files$ ll ~
total 24
drwxr-xr-x 4 ubuntu ubuntu 4096 Jul 24 11:34 ./
drwxr-xr-x 4 root   root   4096 Jul 24 11:31 ../
-rw-r--r-- 1 ubuntu ubuntu  220 Jul 24 11:31 .bash_logout
lrwxrwxrwx 1 ubuntu ubuntu   40 Jul 24 11:34 .bashrc -> /home/ubuntu/dot-files/./homedir/.bashrc
drwxrwxr-x 4 ubuntu ubuntu 4096 Jul 24 11:32 dot-files/
drwxrwxr-x 2 ubuntu ubuntu 4096 Jul 24 11:34 .ff/
-rw-r--r-- 1 ubuntu ubuntu  675 Jul 24 11:31 .profile
```

## Note:
It's easy to replace git (or any other VCS like Mercurial, etc.) with
auto synchronized directory within Dropbox (or other syncing service)
