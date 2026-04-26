import { Button$PressEvent } from "sap/m/Button";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ProblemsController extends BaseController {

  private readonly dataLoader: JSONModel = new JSONModel();
  private readonly problemsModel: JSONModel = new JSONModel();
  private problemsTable?: Table;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("problems")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadProblems(), this);
    super.setViewModel(this.problemsModel, "problems");
    this.problemsTable = this.byId("problemsTable") as Table | undefined;
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    this.setBusy(true);
    this.loadProblems().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => {
      this.setBusy(false);
    });
  }

  private async loadProblems(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const succeeded: boolean = await super.updateJSONModel(this.dataLoader, `/api/regattas/${regatta.id}/races/club-conflicts`);

    if (succeeded) {
      const conflicts: any[] = this.dataLoader.getData() || [];
      // Flatten the nested structure into table rows:
      // Each row = one club conflict in one heat of one race
      const rows: any[] = [];
      for (const race of conflicts) {
        for (const heat of (race.heats || [])) {
          for (const conflict of (heat.conflicts || [])) {
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

  private setBusy(busy: boolean): void {
    this.problemsTable?.setBusy(busy);
  }
}