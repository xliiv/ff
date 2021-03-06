import os
import shutil
import subprocess as subp
import unittest
from pathlib import Path


show_stdout = os.environ.get("FF_TEST_SHOW_STDOUT", "False")
if show_stdout.lower() in ['1', 'true', 'yes']:
    STDOUT = None
else:
    STDOUT = subp.DEVNULL


HOME_DIR = os.path.expanduser("~")
DOT_FILES_DIR = os.path.join(HOME_DIR, "dot-files")
DOT_FILES_SPACE = "homedir"
DOT_FILES_WITH_SPACE = os.path.join(DOT_FILES_DIR, DOT_FILES_SPACE)
FF = "ff"
FF_PATH = os.path.join(DOT_FILES_DIR, FF)


def _setup_test():
    os.makedirs(DOT_FILES_DIR)
    for file_name in [".bashrc", ".bash_history"]:
        Path(
            os.path.join(HOME_DIR, file_name)
        ).touch()
    shutil.copy('/tests/ff', os.path.join(DOT_FILES_DIR, FF))


def _teardown_test():
    try:
        shutil.rmtree(DOT_FILES_DIR)
    except FileNotFoundError as e:
        print(e)
    for file_name in [".bashrc", ".bash_history"]:
        try:
            os.remove(os.path.join(HOME_DIR, file_name))
        except FileNotFoundError as e:
            pass


class Setup:
    def setUp(self):
        _setup_test()
        shutil.copyfile("/tests/ff", os.path.join(DOT_FILES_DIR, FF))
        os.chdir(DOT_FILES_DIR)
        subp.run([FF_PATH, 'init', '--dir-path', '.'], stdout=STDOUT)

    def tearDown(self):
        _teardown_test()


class TestAll(Setup, unittest.TestCase):
    def test_init_works(self):
        self.assertTrue(os.path.exists(os.path.join(HOME_DIR, ".ff/config.ini")))

    def test_add_works(self):
        os.chdir(HOME_DIR)

        subp.run([FF_PATH, 'add', '--file-path', '.bashrc', '--sync-subdir', '.'], stdout=STDOUT)

        self.assertTrue(
            os.path.exists(
                os.path.join(DOT_FILES_DIR, '.bashrc')
            )
        )
        symlinked = os.path.join(HOME_DIR, '.bashrc')
        self.assertTrue(os.path.exists(symlinked))
        self.assertTrue(os.path.islink(symlinked))

    def test_add_works_when_space_passed(self):
        os.chdir(HOME_DIR)

        subp.run([FF_PATH, 'add', '--file-path', '.bashrc', '--sync-subdir', DOT_FILES_SPACE], stdout=STDOUT)

        self.assertTrue(
            os.path.exists(
                os.path.join(DOT_FILES_WITH_SPACE, '.bashrc')
            )
        )
        symlinked = os.path.join(HOME_DIR, '.bashrc')
        self.assertTrue(os.path.exists(symlinked))
        self.assertTrue(os.path.islink(symlinked))

    def test_remove_works(self):
        os.chdir(HOME_DIR)
        subp.run([FF_PATH, 'add', '.bashrc'], stdout=STDOUT)

        subp.run([FF_PATH, 'remove', '.bashrc'], stdout=STDOUT)

        self.assertFalse(
            os.path.exists(
                os.path.join(DOT_FILES_WITH_SPACE, '.bashrc')
            )
        )
        orig_file = os.path.join(HOME_DIR, '.bashrc')
        self.assertTrue(os.path.exists(orig_file))
        self.assertFalse(os.path.islink(orig_file))


class TestApply(Setup, unittest.TestCase):
    def setUp(self):
        super().setUp()
        self.file_name = '.bashrc'
        self.file_to_symlink = os.path.join(DOT_FILES_DIR, self.file_name)
        self.file_symlinked = os.path.join(HOME_DIR, self.file_name)
        Path(self.file_to_symlink).touch()

    def test_apply_works_when_homedir_file_missing(self):
        os.remove(self.file_symlinked)
        self.assertFalse(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply', '--sync-subdir', '.'], stdout=STDOUT)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(os.path.islink(self.file_symlinked))

    def test_apply_works_when_homedir_file_exists(self):
        self.assertTrue(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply', '--sync-subdir', '.'], stdout=STDOUT)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(
            os.path.islink(
                os.path.join(HOME_DIR, self.file_name)
            )
        )


class TestApplyWithSpace(Setup, unittest.TestCase):
    def setUp(self):
        super().setUp()
        os.makedirs(DOT_FILES_WITH_SPACE)
        self.file_name = '.bashrc'
        self.file_to_symlink = os.path.join(DOT_FILES_WITH_SPACE, self.file_name)
        self.file_symlinked = os.path.join(HOME_DIR, self.file_name)
        Path(self.file_to_symlink).touch()

    def test_apply_works_when_homedir_file_missing(self):
        os.remove(self.file_symlinked)
        self.assertFalse(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply', '--sync-subdir', DOT_FILES_SPACE], stdout=STDOUT)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(os.path.islink(self.file_symlinked))

    def test_apply_works_when_homedir_file_exists(self):
        self.assertTrue(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply', '--sync-subdir', DOT_FILES_SPACE], stdout=STDOUT)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(
            os.path.islink(
                os.path.join(HOME_DIR, self.file_name)
            )
        )


class TestApplySkipping(Setup, unittest.TestCase):
    def test_apply_skips_files_in_git_dir(self):
        ignored_dir = os.path.join(DOT_FILES_DIR, '.git')
        os.makedirs(ignored_dir)
        file_to_ignore = 'file-to-ignore'
        ignored_file = os.path.join(ignored_dir, file_to_ignore)
        Path(ignored_file).touch()
        home_dir_file_to_ignore = os.path.join(HOME_DIR, file_to_ignore)
        self.assertTrue(os.path.exists(ignored_file))
        self.assertFalse(os.path.exists(home_dir_file_to_ignore))

        subp.run([FF_PATH, 'apply'], stdout=STDOUT)

        self.assertFalse(os.path.exists(home_dir_file_to_ignore))


if __name__ == '__main__':
    unittest.main()
