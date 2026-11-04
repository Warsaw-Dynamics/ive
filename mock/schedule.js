// IVE mock: scheduled commands (a proposal, DESIGN.md "Scheduled commands"). An agent schedules a command over MCP
// (`schedule_command`): a name, a description, when (once, or repeating), the project, branch, worktree and source
// branch, and the command. When one comes due IVE runs nothing by itself: it opens the Scheduled commands tab, where
// the user reads each command and ticks what runs. Settings › Scheduled lists them all and what became of them.
(function () {
  'use strict';

  var HOUR = 3600 * 1000;
  var DAY = 24 * HOUR;
  // The tab every due command goes to; one for the whole window.
  var TAB_KEY = 'scheduled';
  var CONFIG_FILE = '~/.config/ive/default/schedule.toml';
  // Settings › Scheduled keeps what ran, was skipped or failed this long.
  var HISTORY_DAYS = 7;

  function today() { return window.IVE_SCALE.today; }
  // weekdays at 07:00 (MCP `repeat: "0 7 * * 1-5"`): the next such time after `date`
  var WEEKDAYS_AT_7 = { text: 'weekdays at 07:00', next: function (date) {
    var next = new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1, 7);
    while (next.getDay() === 0 || next.getDay() === 6) next.setDate(next.getDate() + 1);
    return next;
  } };
  function todayAt(hour) { var t = today(); return new Date(t.getFullYear(), t.getMonth(), t.getDate(), hour); }
  function at(hours) { return new Date(today().getTime() + hours * HOUR); }

  // id, name, description, when; target: project (its main checkout's path), branch, worktree (a path, or null for
  // the .ive.toml folder), source (where a missing branch starts); command; repeat: null, or { text, next(date) }.
  // state: pending | due | started | skipped | failed | cancelled; missed: how many runs went by while it waited.
  // A repeating command run with "Always run this one" keeps that exact command in `trusted`: its later times start
  // unasked while the command stays the same (an agent replacing it drops the trust); `always` is the row's tick.
  function schedule(fields) {
    return Object.assign({ description: '', worktree: null, source: null, repeat: null, state: 'pending', missed: 0, process: null, error: null,
      created: at(-26), selected: false, always: false, trusted: null, unasked: false, done: null }, fields);
  }

  function seed() {
    return [
      schedule({ id: 1, name: 'Check CI on GEN-817', created: at(-3),
        description: 'PR #5912 was pushed at 14:05; the pipeline takes about 2 h. Read its result and fix what failed.',
        when: at(-0.5), project: '~/Projects/atlassian', branch: 'GEN-817-ive', source: 'dev',
        command: 'claude --resume 7f3a9c2e-1b4d-4e8a-9f60-2c7d5e1a8b34 "CI on PR #5912 should be done: read the pipeline result and fix failures"' }),
      schedule({ id: 2, name: 'Retry webhook job queue', created: at(-20),
        description: 'GEN-900 was blocked on the queue fix being deployed to staging. Start the follow-up once it is.',
        when: at(-14), project: '~/Projects/atlassian', branch: 'GEN-900-retry-webhooks', source: 'dev',
        command: 'claude "GEN-900: the queue fix is on staging now; rerun the webhook e2e suite and continue the retry work"' }),
      schedule({ id: 3, name: 'Dependency report', repeat: WEEKDAYS_AT_7, created: at(-24 * 12),
        description: 'Posts outdated and vulnerable dependencies to GEN-12.',
        when: todayAt(7), project: '~/Projects/atlassian', branch: 'dev', command: 'node extra/dependency-analyser.js', missed: 2 }),
      schedule({ id: 4, name: 'Release notes for 2.4', created: at(-24 * 5),
        description: 'Draft release notes from the merged PRs once the 2.4 tag exists.',
        when: at(-30), project: '~/Projects/release-tools', branch: 'main', command: './draft-notes.sh v2.4' }),
      schedule({ id: 5, name: 'OIDC login: check the review', created: at(-2),
        description: 'Reviewers were asked on Monday. Answer their comments.',
        when: at(24 * 2 + 3), project: '~/Projects/forge-installer', branch: 'feature/oidc-login', source: 'main',
        command: 'claude "Read the review comments on the OIDC login PR and answer or fix each one"' }),
      schedule({ id: 6, name: 'Dependency report', repeat: WEEKDAYS_AT_7, state: 'started', created: at(-24 * 12),
        when: new Date(todayAt(7) - DAY), done: new Date(todayAt(7) - DAY + 60000), project: '~/Projects/atlassian', branch: 'dev', command: 'node extra/dependency-analyser.js' }),
      schedule({ id: 7, name: 'Bump Forge CLI', state: 'skipped', created: at(-24 * 4),
        when: at(-24 * 3), done: at(-24 * 3 + 1), project: '~/Projects/forge-installer', branch: 'main', command: 'claude "Bump @forge/cli to the latest and run the tests"' })
    ];
  }

  window.IVE_SCHEDULE = {
    state: function () {
      return {
        schedules: seed(),
        // mock-only: the desktop notification IVE sends while in the background (a click shows the tab)
        scheduleNotice: null
      };
    },

    methods: {
      // ---- where a command runs (schedule/target.rs) ----
      scheduleProject: function (s) {
        return this.projects.find(function (p) { return p.path === s.project; }) || null;
      },
      // { co, steps, error }: the checkout it runs in now (null until created), what Run does first, or why it cannot.
      schedulePlan: function (s) {
        var project = this.scheduleProject(s);
        if (project === null) return { co: null, steps: [], error: 'The project ' + s.project + ' is gone' };
        var co = project.checkouts.find(function (c) { return c.branch === s.branch; });
        if (co) return { co: co, steps: [], error: null };
        var branch = project.branches.find(function (b) { return b.name === s.branch; });
        var path = this.worktreePath(project, { name: s.branch });
        if (branch) return { co: null, steps: ['new worktree ' + path], error: null };
        if (s.source === null) return { co: null, steps: [], error: 'No branch ' + s.branch + ' and no source branch to start it from' };
        return { co: null, steps: ['fetch origin', 'new branch ' + s.branch + ' from origin/' + s.source, 'new worktree ' + path], error: null };
      },
      scheduleWhere: function (s) {
        var project = this.scheduleProject(s);
        var name = project ? project.name : s.project;
        return name + ' · ⎇ ' + s.branch;
      },
      scheduleWhereLine: function (s) {
        var plan = this.schedulePlan(s);
        if (plan.error !== null) return plan.error;
        if (plan.co !== null) return 'in ' + plan.co.path + (plan.co.main ? ' (main checkout)' : '');
        return 'Run first: ' + plan.steps.join(', then ');
      },

      // ---- when ----
      // "in 2 d 3 h", "30 min ago"
      scheduleWhen: function (date) {
        var minutes = Math.round((date - today()) / 60000);
        if (minutes === 0) return 'just now';
        var text = function (m) {
          if (m < 60) return m + ' min';
          if (m < 60 * 24) return Math.floor(m / 60) + ' h' + (m % 60 ? ' ' + (m % 60) + ' min' : '');
          return Math.floor(m / 1440) + ' d' + (Math.floor(m % 1440 / 60) ? ' ' + Math.floor(m % 1440 / 60) + ' h' : '');
        };
        return minutes >= 0 ? 'in ' + text(minutes) : text(-minutes) + ' ago';
      },
      scheduleClock: function (date) {
        return date.toLocaleString('en-GB', { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
      },
      scheduleDueLine: function (s) {
        var missed = s.missed ? ' · missed ' + s.missed + ' more time' + (s.missed > 1 ? 's' : '') + ' while waiting' : '';
        var closed = s.closed ? ' · IVE was closed then' : '';
        return 'due ' + this.scheduleWhen(s.when) + ' (' + this.scheduleClock(s.when) + ')' + closed + missed;
      },

      // ---- lists ----
      // the ones that can run first, each part oldest first
      dueSchedules: function () {
        var self = this;
        var rank = function (s) { return self.isRunnable(s) ? 0 : 1; };
        return this.schedules.filter(function (s) { return s.state === 'due'; }).sort(function (a, b) { return rank(a) - rank(b) || a.when - b.when; });
      },
      pendingSchedules: function () { return this.schedules.filter(function (s) { return s.state === 'pending'; }).sort(function (a, b) { return a.when - b.when; }); },
      // started, skipped, failed or cancelled in the last HISTORY_DAYS days, newest first
      scheduleHistory: function () {
        var since = today().getTime() - HISTORY_DAYS * DAY;
        return this.schedules.filter(function (s) { return s.done !== null && s.done.getTime() >= since; }).sort(function (a, b) { return b.done - a.done; });
      },
      // the commands started from the tab while it is open, so the user sees where they went
      startedHere: function () { return this.schedules.filter(function (s) { return s.startedHere; }); },
      isRunnable: function (s) { return this.schedulePlan(s).error === null; },
      selectedSchedules: function () { return this.dueSchedules().filter(function (s) { return s.selected; }); },
      areAllSelected: function () {
        var runnable = this.dueSchedules().filter(this.isRunnable, this);
        return runnable.length > 0 && runnable.every(function (s) { return s.selected; });
      },
      selectAllSchedules: function (on) {
        this.dueSchedules().forEach(function (s) { s.selected = on && this.isRunnable(s); }, this);
      },

      // ---- coming due (schedule/clock.rs) ----
      // The commands whose time came: into the tab, nothing runs. The tab opens without taking the shown tab's place
      // (its dot says why), or shown when IVE starts with commands it missed; in the background a desktop
      // notification says how many, and a click shows the tab.
      comeDue: function (list, how) {
        var unasked = list.filter(this.runsUnasked, this);
        unasked.forEach(this.startUnasked, this);
        list = list.filter(function (s) { return unasked.includes(s) === false; });
        if (list.length === 0) return;
        list.forEach(function (s) { s.state = 'due'; s.selected = false; s.always = false; s.closed = how === 'startup'; });
        this.openScheduledTab(how === 'startup' ? 'show' : 'behind');
        if (this.windowFocused) return;
        var n = this.dueSchedules().length;
        this.scheduleNotice = { title: n === 1 ? 'A scheduled command is due' : n + ' scheduled commands are due',
          text: this.dueSchedules().map(function (s) { return s.name; }).join(', ') + ' · nothing runs until you pick it' };
      },
      openScheduledTab: function (how) {
        var open = this.tabs.find(function (t) { return t.key === TAB_KEY; });
        if (open === undefined) {
          this.leaveScheduledTab();
          var tab = { key: TAB_KEY, kind: 'scheduled', title: 'Scheduled commands', co: null, project: null };
          if (how === 'show') { this.openTab(tab); return; }
          this.tabs.push(tab);
          return;
        }
        if (how === 'show') this.showTab(open);
      },
      showScheduleNotice: function () {
        this.scheduleNotice = null;
        this.windowFocused = true;
        this.openScheduledTab('show');
      },
      // the tab's dot, while commands wait in it and it is not shown
      scheduledAttention: function (tab) {
        if (tab.kind !== 'scheduled' || this.focus === tab.key) return null;
        var n = this.dueSchedules().length;
        return n ? n + (n === 1 ? ' command is' : ' commands are') + ' due' : null;
      },

      // ---- answering ----
      // Run selected: each in a new terminal in its checkout, created first when missing; the tab stays on.
      runSelectedSchedules: function () {
        this.selectedSchedules().forEach(this.startSchedule, this);
      },
      startSchedule: function (s) {
        if (s.always) s.trusted = s.command;
        var c = this.startScheduledTerminal(s);
        s.startedHere = true;
        return c;
      },
      startScheduledTerminal: function (s) {
        var plan = this.schedulePlan(s);
        var co = plan.co || this.createScheduledWorktree(s);
        this.revealWorktree(co);
        var c = this.addConsole(co.id, /^claude\b/.test(s.command) ? 'agent' : 'shell', ['$ ' + s.command]);
        c.title = s.name;
        s.process = c.id;
        this.finishSchedule(s, 'started');
        return c;
      },
      // allowed before for this exact command, and it can run where it is now
      runsUnasked: function (s) { return s.trusted !== null && s.trusted === s.command && this.isRunnable(s); },
      // It starts behind the shown tab; its terminal calls for the user as a program in it would (the dot, the bell,
      // and a desktop notification while IVE is in the background, whose click shows it).
      startUnasked: function (s) {
        var c = this.startScheduledTerminal(s);
        this.lastStarted(s).unasked = true;
        this.raiseAttention(c, 'notify', 'Scheduled: started unasked (' + s.repeat.text + ')');
      },
      // the history entry a repeating command just left, or the command itself
      lastStarted: function (s) {
        var entries = this.schedules.filter(function (x) { return x.name === s.name && x.state === 'started'; });
        return entries[entries.length - 1] || s;
      },
      // Settings › Scheduled: Ask again drops the trust; its next time comes to the tab.
      askAgain: function (s) { s.trusted = null; },
      createScheduledWorktree: function (s) {
        var project = this.scheduleProject(s);
        var branch = project.branches.find(function (b) { return b.name === s.branch; }) || { name: s.branch, remote: null };
        return this.createWorktree(project, branch);
      },
      // Skip selected: this time does not run; a repeating command waits for its next time.
      skipSelectedSchedules: function () {
        this.selectedSchedules().forEach(function (s) { this.finishSchedule(s, 'skipped'); }, this);
      },
      // A command that cannot run (its project is gone) goes with Remove; it is in the history as failed.
      removeFailedSchedule: function (s) {
        s.error = this.schedulePlan(s).error;
        this.finishSchedule(s, 'failed');
      },
      // A repeating command leaves a history entry and goes on pending at its next time.
      finishSchedule: function (s, state) {
        s.selected = false;
        if (s.repeat !== null) {
          this.schedules.push(Object.assign({}, s, { id: this.schedules.length + 100, state: state, done: today(), missed: 0, startedHere: false, unasked: false }));
          s.when = s.repeat.next(today());
          s.missed = 0;
          s.state = 'pending';
          return;
        }
        s.state = state;
        s.done = today();
      },
      showScheduledProcess: function (s) {
        var c = this.consoles.find(function (x) { return x.id === s.process; });
        if (c) this.openConsole(c);
      },
      // Settings › Scheduled: Cancel takes a pending command off (a repeating one with every later time).
      cancelSchedule: function (s) {
        s.state = 'cancelled';
        s.done = today();
      },
      // a new Scheduled commands tab lists no starts from an earlier one; due commands stay due when it closes
      leaveScheduledTab: function () {
        this.schedules.forEach(function (s) { s.startedHere = false; });
      },
      scheduleStateText: function (s) {
        var words = { started: 'started', skipped: 'skipped', failed: 'failed', cancelled: 'cancelled', due: 'due', pending: 'pending' };
        return words[s.state];
      },
      scheduleConfigFile: function () { return CONFIG_FILE; },

      // ---- mock-only: the Mock menu ----
      // Two commands come due while IVE runs.
      mockScheduleDue: function () {
        var list = this.schedules.filter(function (s) { return s.id === 1 || s.id === 3; });
        list.forEach(function (s) { s.when = s.repeat ? todayAt(7) : at(-0.1); });
        this.comeDue(list, 'running');
      },
      // IVE starts with four commands it missed while closed: one runs in place, one needs a worktree, one a new
      // branch, one has lost its project.
      mockScheduleStartup: function () {
        this.comeDue(this.schedules.filter(function (s) { return [1, 2, 3, 4].includes(s.id); }), 'startup');
      }
    }
  };
})();
