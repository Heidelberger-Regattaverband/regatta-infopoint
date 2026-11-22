import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import ListBinding from "sap/ui/model/ListBinding";
import BaseTableController from "./BaseTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubsTableController extends BaseTableController {
  private static readonly CLUBS_MODEL: string = "clubs";

  onInit(): void {
    super.init(super.getView()?.byId("clubsTable") as Table);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), ClubsTableController.CLUBS_MODEL);
    super
      .getRouter()
      ?.getRoute("clubs")
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadModel(), "Failed to load clubs:");
      }, this);

    super.sortTable("clubCityCol", false, "city");
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding.filter(searchFilters);
  }

  onSortButtonPress(_event: Button$PressEvent): void {
    super.runAsync(
      super.getViewSettingsDialog("de.regatta_hd.infoportal.view.ClubsTableSortDialog").then((dialog) => dialog.open()),
      "Failed to open club sort dialog:"
    );
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadModel()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh clubs:"
    );
  }

  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("clubs");
      const club: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.navToClubDetails(club.id);
    }
  }

  protected onItemChanged(_item: any): void {
    // nothing to do yet
  }

  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("shortName", FilterOperator.Contains, query),
          new Filter("longName", FilterOperator.Contains, query),
          new Filter("abbreviation", FilterOperator.Contains, query),
          new Filter("city", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  private async loadModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url: string = `/api/regattas/${regatta.id}/clubs`;
    const clubsModel: JSONModel = super.getViewJSONModel(ClubsTableController.CLUBS_MODEL);
    return await super.updateJSONModel(clubsModel, url);
  }
}
