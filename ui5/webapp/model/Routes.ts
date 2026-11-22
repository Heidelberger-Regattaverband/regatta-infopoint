/**
 * Named routes declared in `sap.ui5.routing.routes` in `manifest.json`.
 *
 * Use these values for router lookups and navigation, not as URL patterns or model names.
 * `as const` preserves literal types and readonly properties at compile time; it does not freeze the object at runtime.
 *
 * @example
 * ```ts
 * router.navTo(Routes.AthleteDetails, { athleteId: 123 });
 * ```
 */
export const Routes = {
  StartPage: "startpage",
  Races: "races",
  RaceDetails: "raceDetails",
  Heats: "heats",
  HeatDetails: "heatDetails",
  Clubs: "clubs",
  ClubDetails: "clubDetails",
  Athletes: "athletes",
  ClubAthletes: "clubAthletes",
  AthleteDetails: "athleteDetails",
  MapClub: "mapClub",
  Map: "map",
  Schedule: "schedule",
  Scoring: "scoring",
  Statistics: "statistics",
  Monitoring: "monitoring",
  Timekeeping: "timekeeping",
  Problems: "problems",
  Notifications: "notifications"
} as const;

/** Union of route-name values, used to constrain navigation targets. */
export type RouteName = (typeof Routes)[keyof typeof Routes];
