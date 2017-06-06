import os
import shutil
import subprocess as subp
import unittest
from pathlib import Path


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
        subp.run([FF_PATH, 'init', '.'], stdout=subp.DEVNULL)

    def tearDown(self):
        _teardown_test()


class TestAll(Setup, unittest.TestCase):
    def test_init_works(self):
        self.assertTrue(os.path.exists(os.path.join(HOME_DIR, ".ff/config.ini")))

    def test_add_works(self):
        os.chdir(HOME_DIR)

        subp.run([FF_PATH, 'add', '.bashrc'], stdout=subp.DEVNULL)

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
        subp.run([FF_PATH, 'add', '.bashrc'], stdout=subp.DEVNULL)

        subp.run([FF_PATH, 'remove', '.bashrc'], stdout=subp.DEVNULL)

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
        os.makedirs(DOT_FILES_SPACE)
        self.file_name = '.bashrc'
        self.file_to_symlink = os.path.join(DOT_FILES_SPACE, self.file_name)
        self.file_symlinked = os.path.join(HOME_DIR, self.file_name)
        Path(self.file_to_symlink).touch()

    def test_apply_works_When_homedir_file_missing(self):
        os.remove(self.file_symlinked)
        self.assertFalse(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply'], stdout=subp.DEVNULL)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(os.path.islink(self.file_symlinked))

    def test_apply_works_When_homedir_file_exists(self):
        self.assertTrue(os.path.exists(self.file_symlinked))

        subp.run([FF_PATH, 'apply'], stdout=subp.DEVNULL)

        self.assertFalse(os.path.islink(self.file_to_symlink))
        self.assertTrue(
            os.path.islink(
                os.path.join(HOME_DIR, self.file_name)
            )
        )


if __name__ == '__main__':
    unittest.main()