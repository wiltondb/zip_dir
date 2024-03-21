/*
 * Copyright 2024, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn read_dir(dir: &Path) -> Vec<String> {
    let rd = fs::read_dir(dir).unwrap();
    let mut paths: Vec<PathBuf> = Vec::new();
    for en in rd {
        let en = en.unwrap();
        paths.push(en.path());
    }
    paths.sort_by(|a, b| {
        if a.is_dir() && !b.is_dir() {
            std::cmp::Ordering::Less
        } else if b.is_dir() && !a.is_dir() {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });
    let res = paths.iter().map(|p| {
        p.file_name().unwrap().to_string_lossy().to_string()
    }).collect();

    res
}

#[test]
fn zip_test() {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let work_dir = project_dir.join("target/zip_test");
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir).unwrap();
    }
    fs::create_dir(&work_dir).unwrap();

    let orig_dir = work_dir.join("test");
    fs::create_dir(orig_dir.as_path()).unwrap();
    fs::write(orig_dir.join("foo.txt"), "foo").unwrap();
    fs::write(orig_dir.join("bar.txt"), "bar").unwrap();
    let orig_baz_dir = orig_dir.join("baz");
    fs::create_dir(orig_baz_dir.as_path()).unwrap();
    fs::write(orig_baz_dir.join("boo.txt"), "boo").unwrap();

    let expected_entries = vec!(
        "test/",
        "test/baz/",
        "test/baz/boo.txt",
        "test/bar.txt",
        "test/foo.txt",
    );

    // store
    let stored_dir = work_dir.join("stored");
    fs::create_dir(&stored_dir).unwrap();
    let stored_file = stored_dir.join("test.zip");
    let mut stored_entries = Vec::new();
    zip_recurse::zip_directory_listen(&orig_dir, &stored_file, 0, |en: &str| {
        stored_entries.push(en.to_string());
    }).unwrap();
    assert_eq!(expected_entries, stored_entries);

    // deflate
    let deflated_dir = work_dir.join("deflated");
    fs::create_dir(&deflated_dir).unwrap();
    let deflated_file = deflated_dir.join("test.zip");
    zip_recurse::zip_directory(&orig_dir, &deflated_file, 6).unwrap();

    // unzip
    let stored_unzipped = stored_dir.join("unzipped");
    let root_dir = zip_recurse::unzip_directory(&stored_file, &stored_unzipped).unwrap();
    assert_eq!("test/", root_dir);
    let deflated_unzipped = deflated_dir.join("unzipped");
    let mut deflated_entries = Vec::new();
    zip_recurse::unzip_directory_listen(&deflated_file, &deflated_unzipped, |en: &str| {
        deflated_entries.push(en.to_string());
    }).unwrap();
    assert_eq!(expected_entries, deflated_entries);

    assert_eq!(vec!("baz", "bar.txt", "foo.txt"), read_dir(&stored_unzipped.join("test")));
    assert_eq!(vec!("boo.txt"), read_dir(&stored_unzipped.join("test").join("baz")));

    let du_root = deflated_unzipped.join("test");
    assert_eq!(fs::read_to_string(du_root.join("foo.txt")).unwrap(), "foo");
    assert_eq!(fs::read_to_string(du_root.join("bar.txt")).unwrap(), "bar");
    assert_eq!(fs::read_to_string(du_root.join("baz").join("boo.txt")).unwrap(), "boo");
}
