use chrono;
use todo_lib::{tfilter, todo, todotxt, tsort};

fn init_tasks() -> todo::TaskVec {
    let mut t = Vec::new();
    let now = chrono::Local::now().date().naive_local();

    t.push(todotxt::Task::parse("call mother +family @parents", now));
    t.push(todotxt::Task::parse(
        "x (C) 2018-10-05 2018-10-01 call to car service and schedule repair +car @repair",
        now,
    ));
    t.push(todotxt::Task::parse("(B) 2018-10-15 repair family car +Car @repair due:2018-12-01 t:2019-01-02", now));
    t.push(todotxt::Task::parse("(A) Kid's art school lesson +Family @Kids due:2018-11-10 rec:1w", now));
    t.push(todotxt::Task::parse("take kid to hockey game +Family @kids due:2018-11-18", now));
    t.push(todotxt::Task::parse("xmas vacations +FamilyHoliday due:2018-12-24", now));

    t
}

#[test]
fn one_item() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // invalid ranges
    cflt.range = tfilter::ItemRange::One(t.len());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    cflt.range = tfilter::ItemRange::One(t.len() + 1);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);
}

#[test]
fn item_range() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // both ends are out of range
    cflt.range = tfilter::ItemRange::Range(t.len(), t.len() + 5);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    // one item that is completed
    cflt.range = tfilter::ItemRange::One(1);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    // short range only active
    cflt.range = tfilter::ItemRange::Range(1, 3);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3]);
    cflt.range = tfilter::ItemRange::None;
}

#[test]
fn item_status() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // one incomplete
    cflt.range = tfilter::ItemRange::One(0);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0]);

    // full range all
    cflt.all = tfilter::TodoStatus::All;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 3, 4, 5]);

    // full range only completed
    cflt.all = tfilter::TodoStatus::Done;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1]);

    // full range only active
    cflt.all = tfilter::TodoStatus::Active;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 2, 3, 4, 5]);
}

#[test]
fn item_regex() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // all with 'car' anywhere
    cflt.all = tfilter::TodoStatus::All;
    cflt.regex = Some("car".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1, 2]);
    cflt.all = tfilter::TodoStatus::Active;

    // active with <regex> anywhere
    cflt.use_regex = true;
    cflt.regex = Some("CA[rl]".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 2]);
}

#[test]
fn item_projects() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // active with 'car' project
    cflt.include.projects.push("car".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);
    cflt.include.projects.clear();

    // active with 'family' project
    cflt.include.projects.push("FAMILY".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4]);
    cflt.include.projects.clear();

    // active with 'family' project
    cflt.include.projects.push("FAMILY*".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4, 5]);
    cflt.include.projects.clear();

    // active with 'holiday' project
    cflt.include.projects.push("*holiday".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![5]);
    cflt.include.projects.clear();

    // active with 'family' related projects
    cflt.include.projects.push("*family*".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4, 5]);
    cflt.include.projects.clear();

    // Exclude filtering
    cflt.exclude.projects.push("*family*".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);
    cflt.exclude.projects.clear();

    cflt.exclude.projects.push("FAMILY".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 5]);
    cflt.exclude.projects.clear();

    // Exclude has higher priority
    cflt.include.projects.push("*family*".to_owned());
    cflt.exclude.projects.push("FAMILY".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![5]);
}

#[test]
fn item_contexts() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // active with 'kids' context
    cflt.include.contexts.push("kids".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![3, 4]);
    cflt.include.contexts.clear();

    cflt.exclude.contexts.push("kids".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 2, 5]);
}

#[test]
fn item_priority() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // only B priority
    cflt.pri = Some(tfilter::Priority { value: 'b' as u8 - 'a' as u8, span: tfilter::ValueSpan::Equal });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);

    // B priority and higher
    cflt.pri = Some(tfilter::Priority { value: 'b' as u8 - 'a' as u8, span: tfilter::ValueSpan::Higher });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3]);

    //  B priority and lower
    cflt.pri = Some(tfilter::Priority { value: 'b' as u8 - 'a' as u8, span: tfilter::ValueSpan::Lower });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 4, 5]);

    // any priority except no priority
    cflt.pri = Some(tfilter::Priority { value: todotxt::NO_PRIORITY, span: tfilter::ValueSpan::Any });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1, 2, 3]);

    // no priority
    cflt.pri = Some(tfilter::Priority { value: todotxt::NO_PRIORITY, span: tfilter::ValueSpan::None });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 4, 5]);
}

#[test]
fn item_recurrence() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with recurrence
    cflt.rec = Some(tfilter::Recurrence { span: tfilter::ValueSpan::Any });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![3]);

    // without recurrence
    cflt.rec = Some(tfilter::Recurrence { span: tfilter::ValueSpan::None });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 4, 5]);
}

#[test]
fn item_due() {
    let t = init_tasks();

    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with due
    cflt.due = Some(tfilter::DateRange { span: tfilter::ValueSpan::Any, days: Default::default() });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3, 4, 5]);

    // without due
    cflt.due = Some(tfilter::DateRange { span: tfilter::ValueSpan::None, days: Default::default() });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1]);

    let sconf = tsort::Conf { fields: Some("due".to_string()), rev: true };
    let mut ids: todo::IDVec = vec![0, 1, 2, 3, 4, 5];
    tsort::sort(&mut ids, &t, &sconf);
    assert_eq!(ids, vec![1, 0, 5, 2, 4, 3]);
}

#[test]
fn item_threshold() {
    let t = init_tasks();

    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with thr
    cflt.thr = Some(tfilter::DateRange { span: tfilter::ValueSpan::Any, days: Default::default() });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);

    // without thr
    cflt.thr = Some(tfilter::DateRange { span: tfilter::ValueSpan::None, days: Default::default() });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 3, 4, 5]);

    let sconf = tsort::Conf { fields: Some("thr".to_string()), rev: false };
    let mut ids: todo::IDVec = vec![0, 1, 2, 3, 4, 5];
    tsort::sort(&mut ids, &t, &sconf);
    assert_eq!(ids, vec![2, 0, 1, 3, 4, 5]);
}
