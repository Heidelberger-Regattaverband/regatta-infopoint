import Table from "sap/m/Table";
import JSONModel from "sap/ui/model/json/JSONModel";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Button, { Button$PressEvent } from "sap/m/Button";
import Context from "sap/ui/model/Context";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import MessageToast from "sap/m/MessageToast";
import BaseTableController from "./BaseTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ParticipatingClubsTable extends BaseTableController {

  private participatingClubsModel: JSONModel = new JSONModel();

  onInit(): void {
    super.init(super.getView()?.byId("clubsTable") as Table, "club" /* eventBus channel */);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.participatingClubsModel, "clubs");
    super.getRouter()?.getRoute("participatingClubs")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadModel(), this);
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

  async onSortButtonPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.ParticipatingClubsSortDialog")).open();
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    const updated: boolean = await this.loadModel();
    if (updated) {
      MessageToast.show(this.i18n("msg.dataUpdated"));
    }
    source.setEnabled(true);
  }

  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("clubs");
      const club: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.getRouter().navTo("clubParticipations", { clubId: club.id }, false /* history*/);
    }
  }

  onItemChanged(item: any): void {
    // nothing to do yet
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("shortName", FilterOperator.Contains, query),
        new Filter("longName", FilterOperator.Contains, query),
        new Filter("abbreviation", FilterOperator.Contains, query),
        new Filter("city", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private async loadModel(): Promise<boolean> {
    return await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${this.getRegattaId()}/participating_clubs`, this.table)
  }

}