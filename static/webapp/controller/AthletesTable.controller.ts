import Button, { Button$PressEvent } from "sap/m/Button";
import { ComboBox$SelectionChangeEvent } from "sap/m/ComboBox";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseTableController from "./BaseTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AthletesTable extends BaseTableController {

  private static readonly ATHLETES_MODEL: string = "athletes";
  private static readonly CLUB_FILTER_MODEL: string = "clubFilter";

  private clubFilter?: Filter;
  private textSearchFilters: Filter[] = [];

  onInit(): void {
    super.init(super.getView()?.byId("athletesTable") as Table);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), AthletesTable.ATHLETES_MODEL);
    super.setViewModel(new JSONModel([]), AthletesTable.CLUB_FILTER_MODEL);
    super.getRouter()?.getRoute("athletes")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadModel(), this);

    super.sortTable("athleteLastNameCol", false, "lastName");
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    this.textSearchFilters = query ? this.createSearchFilters(query) : [];
    this.applyAllFilters();
  }

  onClubSelectionChange(event: ComboBox$SelectionChangeEvent): void {
    const key: string = event.getParameters().selectedItem?.getKey() ?? "";
    this.clubFilter = key ? new Filter("club/shortName", FilterOperator.EQ, key) : undefined;
    this.applyAllFilters();
  }

  async onSortButtonPress(event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.AthletesSortDialog");
      dialog.open();
    } catch (error) { console.error("Failed to load sort dialog:", error); }
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: boolean = await this.loadModel();
      super.showDataUpdatedMessage(succeeded);
    } catch (error) {
      console.error("Failed to refresh athletes:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("athletes");
      const athlete: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.navToAthleteDetails(athlete.id);
    }
  }

  onItemChanged(item: any): void {
    // nothing to do yet
  }

  private applyAllFilters(): void {
    const allFilters: Filter[] = [...this.textSearchFilters];
    if (this.clubFilter) {
      allFilters.push(this.clubFilter);
    }
    super.setSearchFilters(allFilters);
    super.applyFilters();
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("lastName", FilterOperator.Contains, query),
        new Filter("firstName", FilterOperator.Contains, query),
        new Filter("year", FilterOperator.EQ, query),
        new Filter("club/shortName", FilterOperator.Contains, query),
        new Filter("club/longName", FilterOperator.Contains, query),
        new Filter("club/abbreviation", FilterOperator.Contains, query),
        new Filter("club/city", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private populateClubFilterModel(athletesModel: JSONModel): void {
    const data: any = athletesModel.getData();
    const athletes: any[] = Array.isArray(data) ? data : [];
    const seen = new Set<string>();
    const items: { key: string; text: string }[] = [];

    athletes
      .map((a: any) => a.club)
      .filter((club: any) => club?.shortName)
      .sort((a: any, b: any) => (a.shortName as string).localeCompare(b.shortName as string))
      .forEach((club: any) => {
        if (!seen.has(club.shortName as string)) {
          seen.add(club.shortName as string);
          items.push({ key: club.shortName as string, text: club.shortName as string });
        }
      });

    super.getViewJSONModel(AthletesTable.CLUB_FILTER_MODEL).setData(items);
  }

  private async loadModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url = `/api/regattas/${regatta.id}/athletes`;
    const athletesModel: JSONModel = super.getViewJSONModel(AthletesTable.ATHLETES_MODEL);
    const succeeded: boolean = await super.updateJSONModel(athletesModel, url);

    if (succeeded) {
      this.populateClubFilterModel(athletesModel);
    }

    return succeeded;
  }
}
