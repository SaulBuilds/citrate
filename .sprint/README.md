# Sprint Planning Directory

This folder contains all sprint planning, tracking, and retrospective documentation for the Citrate AI-First GUI development initiative.

## Structure

```
.sprint/
├── README.md                    # This file
├── ROADMAP.md                   # Master roadmap with all phases
├── CURRENT_SPRINT.md            # Active sprint details (symlink or current)
├── BACKLOG.md                   # Product backlog (prioritized)
├── sprints/
│   ├── sprint-00-critical-fixes/
│   │   ├── SPRINT.md            # Sprint goals, WBS, acceptance criteria
│   │   ├── DAILY.md             # Daily standups/progress
│   │   └── RETRO.md             # Sprint retrospective
│   ├── sprint-01-agent-foundation/
│   ├── sprint-02-frontend-redesign/
│   └── ...
└── templates/
    ├── SPRINT_TEMPLATE.md
    ├── DAILY_TEMPLATE.md
    └── RETRO_TEMPLATE.md
```

## Agile Methodology

We use a modified Scrum approach with:
- **2-week sprints** (10 working days)
- **Work Breakdown Structure (WBS)** for task decomposition
- **Work Packages (WP)** with clear deliverables
- **Story points** using Fibonacci (1, 2, 3, 5, 8, 13)
- **Definition of Done** per work package

## Sprint Workflow

1. **Sprint Planning** (Day 0)
   - Review ROADMAP.md for phase goals
   - Pull items from BACKLOG.md
   - Create detailed SPRINT.md with WBS

2. **Daily Work** (Days 1-10)
   - Update DAILY.md with progress
   - Mark WPs as: `[ ]` pending, `[~]` in progress, `[x]` done

3. **Sprint Review** (Day 10)
   - Demo completed work
   - Update CURRENT_SPRINT.md status

4. **Retrospective** (Day 10)
   - Fill out RETRO.md
   - Identify improvements for next sprint

## Quick Commands

```bash
# View current sprint
cat .sprint/CURRENT_SPRINT.md

# View roadmap
cat .sprint/ROADMAP.md

# Check backlog
cat .sprint/BACKLOG.md
```

## Version

- **Initiative**: AI-First GUI Transformation
- **Start Date**: December 2024
- **Target Completion**: March 2025
