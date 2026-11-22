import { Button$PressEvent } from "sap/m/Button";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Routes } from "../model/Routes";
import BaseController from "./Base.controller";

/**
 * Club-conflict view displaying one table row per conflict within a heat of a race.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ProblemsController extends BaseController {
  private readonly dataLoader: JSONModel = new JSONModel();
  private readonly problemsModel: JSONModel = new JSONModel();
  private problemsTable?: Table;

  /** Registers the conflict display model and loads it when the problems route matches. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super
      .getRouter()
      ?.getRoute(Routes.Problems)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadProblems(), "Failed to load problems:");
      }, this);
    super.setViewModel(this.problemsModel, "problems");
    this.problemsTable = this.byId("problemsTable") as Table | undefined;
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  /**
   * Refreshes conflicts with a table busy indicator that clears even when loading rejects.
   *
   * @param _event - Refresh-button press event; unused because busy state is applied to the table.
   */
  onRefreshButtonPress(_event: Button$PressEvent): void {
    this.setBusy(true);
    super.runAsync(
      this.loadProblems()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => {
          this.setBusy(false);
        }),
      "Failed to refresh problems:"
    );
  }

  /**
   * Flattens nested race/heat/club conflicts for display, joining bib numbers and clearing rows on request failure.
   *
   * @returns Whether the conflict request succeeded; active-regatta loading failures still reject.
   */
  private async loadProblems(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const succeeded: boolean = await super.updateJSONModel(
      this.dataLoader,
      `/api/regattas/${regatta.id}/races/club-conflicts`
    );

    if (succeeded) {
      const conflicts: any[] = this.dataLoader.getData() || [];
      // Flatten the nested structure into table rows:
      // Each row = one club conflict in one heat of one race
      const rows: any[] = [];
      for (const race of conflicts) {
        for (const heat of race.heats || []) {
          for (const conflict of heat.conflicts || []) {
            rows.push({
              raceId: race.raceId,
              raceNumber: race.raceNumber,
              raceShortLabel: race.raceShortLabel,
              raceLongLabel: race.raceLongLabel,
              heatNumber: heat.heatNumber,
              clubId: conflict.clubId,
              clubName: conflict.clubName,
              bibs: conflict.bibs.join(", ")
            });
          }
        }
      }
      this.problemsModel.setData(rows);
    } else {
      this.problemsModel.setData([]);
    }

    return succeeded;
  }

  /**
   * Updates the conflict table's busy indicator if the table is available.
   *
   * @param busy - Whether the table should display its busy indicator.
   */
  private setBusy(busy: boolean): void {
    this.problemsTable?.setBusy(busy);
  }
}
